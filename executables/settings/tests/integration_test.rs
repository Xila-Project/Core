extern crate alloc;

use executable::Standard;
use file_system::{MemoryDevice, Mode, create_device, create_file_system};
use settings::SettingsExecutable;
use task::test;

#[cfg(target_os = "linux")]
#[ignore]
#[test]
async fn main() {
    use alloc::string::ToString;
    use command_line_shell::ShellExecutable;
    use drivers::native::window_screen;
    use executable::mount_static_executables;
    use graphics::{InputKind, Point, get_minimal_buffer_size};
    use virtual_file_system::{create_default_hierarchy, mount_static_devices};

    // - Initialize the task manager.
    let task_manager = task::initialize();

    let task = task_manager.get_current_task_identifier().await;

    // - Initialize the user manager.
    let _ = users::initialize();

    // - Initialize the time manager.
    let _ = time::initialize(create_device!(drivers::native::TimeDevice::new()));

    // - Initialize the virtual file system.
    let memory_device = create_device!(MemoryDevice::<512>::new(1024 * 512));

    little_fs::FileSystem::format(memory_device.clone(), 256).unwrap();

    let file_system = little_fs::FileSystem::new(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(create_file_system!(file_system), None).unwrap();

    // - Initialize the graphics manager.

    const RESOLUTION: Point = Point::new(800, 480);

    let (screen_device, pointer_device, keyboard_device) = window_screen::new(RESOLUTION).unwrap();

    const BUFFER_SIZE: usize = get_minimal_buffer_size(&RESOLUTION);

    let graphics_manager = graphics::initialize(
        screen_device,
        pointer_device,
        InputKind::Pointer,
        BUFFER_SIZE,
        true,
    )
    .await;

    graphics_manager
        .add_input_device(keyboard_device, InputKind::Keypad)
        .await
        .unwrap();

    task_manager
        .spawn(task, "Graphics", None, |_| {
            graphics_manager.r#loop(task::Manager::sleep)
        })
        .await
        .unwrap();

    create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    mount_static_devices!(
        virtual_file_system,
        task,
        &[
            (
                &"/devices/standard_in",
                drivers::standard_library::console::StandardInDevice
            ),
            (
                &"/devices/standard_out",
                drivers::standard_library::console::StandardOutDevice
            ),
            (
                &"/devices/standard_error",
                drivers::standard_library::console::StandardErrorDevice
            ),
            (&"/devices/time", drivers::native::TimeDevice),
            (&"/devices/random", drivers::native::RandomDevice),
            (&"/devices/null", drivers::core::NullDevice)
        ]
    )
    .await
    .unwrap();

    mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (&"/binaries/command_line_shell", ShellExecutable),
            (&"/binaries/settings", SettingsExecutable)
        ]
    )
    .await
    .unwrap();

    let standard_in = virtual_file_system
        .open(&"/devices/standard_in", Mode::READ_ONLY.into(), task)
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(&"/devices/standard_out", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(&"/devices/standard_error", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard = Standard::new(
        standard_in,
        standard_out,
        standard_error,
        task,
        virtual_file_system,
    );

    task_manager
        .set_environment_variables(task, &[("Paths", "/"), ("Host", "xila")])
        .await
        .unwrap();

    let result = executable::execute("/binaries/settings", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert_eq!(result, 0);
}
