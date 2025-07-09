#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use executable::Standard_type;
use file_system::{create_device, Create_file_system, Memory_device_type, Mode_type};
use settings::Settings_executable_type;
use task::Test;

#[cfg(target_os = "linux")]
#[ignore]
#[Test]
async fn main() {
    use alloc::string::ToString;
    use command_line_shell::Shell_executable_type;
    use drivers::native::window_screen;
    use executable::Mount_static_executables;
    use graphics::{get_minimal_buffer_size, Input_type_type, Point_type};
    use virtual_file_system::{create_default_hierarchy, Mount_static_devices};

    // - Initialize the task manager.
    let task_instance = task::initialize();

    // - Initialize the user manager.
    let _ = users::initialize();

    // - Initialize the time manager.
    let _ = time::initialize(create_device!(drivers::native::Time_driver_type::new()));

    // - Initialize the virtual file system.
    let memory_device = create_device!(Memory_device_type::<512>::new(1024 * 512));

    little_fs::File_system_type::format(memory_device.clone(), 256).unwrap();

    let file_system = little_fs::File_system_type::new(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(Create_file_system!(file_system), None).unwrap();

    // - Initialize the graphics manager.

    const RESOLUTION: Point_type = Point_type::new(800, 480);

    let (screen_device, pointer_device, keyboard_device) = window_screen::new(RESOLUTION).unwrap();

    const BUFFER_SIZE: usize = get_minimal_buffer_size(&RESOLUTION);

    graphics::initialize(
        screen_device,
        pointer_device,
        Input_type_type::Pointer,
        BUFFER_SIZE,
        true,
    )
    .await;

    graphics::get_instance()
        .add_input_device(keyboard_device, Input_type_type::Keypad)
        .await
        .unwrap();

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
                drivers::standard_library::console::Standard_in_device_type
            ),
            (
                &"/Devices/Standard_out",
                drivers::standard_library::console::Standard_out_device_type
            ),
            (
                &"/Devices/Standard_error",
                drivers::standard_library::console::Standard_error_device_type
            ),
            (&"/Devices/Time", drivers::native::Time_driver_type),
            (&"/Devices/Random", drivers::native::Random_device_type),
            (&"/Devices/Null", drivers::core::Null_device_type)
        ]
    )
    .await
    .unwrap();

    Mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (&"/Binaries/Command_line_shell", Shell_executable_type),
            (&"/Binaries/Settings", Settings_executable_type)
        ]
    )
    .await
    .unwrap();

    let standard_in = virtual_file_system
        .open(&"/Devices/Standard_in", Mode_type::READ_ONLY.into(), task)
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(&"/Devices/Standard_out", Mode_type::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(
            &"/Devices/Standard_error",
            Mode_type::WRITE_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard = Standard_type::new(
        standard_in,
        standard_out,
        standard_error,
        task,
        virtual_file_system,
    );

    task_instance
        .Set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_instance
        .Set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();

    let result = executable::execute("/Binaries/Settings", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert_eq!(result, 0);
}
