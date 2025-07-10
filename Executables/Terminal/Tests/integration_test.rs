#![no_std]

extern crate alloc;

use executable::Standard;
use file_system::{create_device, Create_file_system, MemoryDeviceType, Mode};
use task::test;
use terminal::TerminalExecutableType;

#[cfg(target_os = "linux")]
#[ignore]
#[test]
async fn main() {
    use alloc::string::ToString;
    use command_line_shell::ShellExecutable;
    use drivers::native::window_screen;
    use executable::mount_static_executables;
    use graphics::{get_minimal_buffer_size, InputKind, Point};
    use virtual_file_system::{create_default_hierarchy, Mount_static_devices};

    log::initialize(&drivers::standard_library::log::LoggerType).unwrap();

    // - Initialize the task manager.
    let task_instance = task::initialize();

    // - Initialize the user manager.
    let _ = users::initialize();

    // - Initialize the time manager.
    let _ = time::initialize(create_device!(drivers::native::TimeDriverType::new()));

    // - Initialize the graphics manager.

    const RESOLUTION: Point = Point::new(800, 480);

    let (screen_device, pointer_device, keyboard_device) = window_screen::new(RESOLUTION).unwrap();

    const BUFFER_SIZE: usize = get_minimal_buffer_size(&RESOLUTION);

    graphics::initialize(
        screen_device,
        pointer_device,
        InputKind::Pointer,
        BUFFER_SIZE,
        true,
    )
    .await;

    graphics::get_instance()
        .add_input_device(keyboard_device, InputKind::Keypad)
        .await
        .unwrap();

    // - Initialize the virtual file system.
    let memory_device = create_device!(MemoryDeviceType::<512>::new(1024 * 512));

    little_fs::FileSystem::format(memory_device.clone(), 256).unwrap();

    let file_system = little_fs::FileSystem::new(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(Create_file_system!(file_system), None).unwrap();

    let task = task_instance.get_current_task_identifier().await;

    create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    Mount_static_devices!(
        virtual_file_system,
        task,
        &[
            (
                &"/Devices/Standard_in",
                drivers::standard_library::console::StandardInDevice
            ),
            (
                &"/Devices/Standard_out",
                drivers::standard_library::console::StandardOutDeviceType
            ),
            (
                &"/Devices/Standard_error",
                drivers::standard_library::console::StandardErrorDeviceType
            ),
            (&"/Devices/Time", drivers::native::TimeDriverType),
            (&"/Devices/Random", drivers::native::RandomDeviceType),
            (&"/Devices/Null", drivers::core::NullDeviceType)
        ]
    )
    .await
    .unwrap();

    mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (&"/Binaries/Command_line_shell", ShellExecutable),
            (&"/Binaries/Terminal", TerminalExecutableType)
        ]
    )
    .await
    .unwrap();

    let standard_in = virtual_file_system
        .open(&"/Devices/Standard_in", Mode::READ_ONLY.into(), task)
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(&"/Devices/Standard_out", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(&"/Devices/Standard_error", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard = Standard::new(
        standard_in,
        standard_out,
        standard_error,
        task,
        virtual_file_system,
    );

    task_instance
        .set_environment_variable(task, "User", "xila")
        .await
        .unwrap();

    task_instance
        .set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_instance
        .set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();

    let result = executable::execute("/Binaries/Terminal", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert_eq!(result, 0);
}
