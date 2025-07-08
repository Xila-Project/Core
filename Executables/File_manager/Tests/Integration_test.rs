#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use Executable::Standard_type;
use File_manager::File_manager_executable_type;
use File_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use Task::Test;

#[cfg(target_os = "linux")]
#[ignore]
#[Test]
async fn main() {
    use alloc::string::ToString;
    use Command_line_shell::Shell_executable_type;
    use Drivers::Native::Window_screen;
    use Executable::Mount_static_executables;
    use Graphics::{Get_minimal_buffer_size, Input_type_type, Point_type};
    use Virtual_file_system::{create_default_hierarchy, Mount_static_devices};

    // - Initialize the task manager.
    let task_instance = Task::Initialize();

    // - Initialize the user manager.
    let _ = Users::Initialize();

    // - Initialize the time manager.
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::new()));

    // - Initialize the virtual file system.
    let memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::format(memory_device.clone(), 256).unwrap();

    let file_system = LittleFS::File_system_type::new(memory_device, 256).unwrap();

    let virtual_file_system =
        Virtual_file_system::initialize(Create_file_system!(file_system), None).unwrap();

    // - Initialize the graphics manager.

    const RESOLUTION: Point_type = Point_type::new(800, 480);

    let (screen_device, pointer_device, keyboard_device) = Window_screen::New(RESOLUTION).unwrap();

    const BUFFER_SIZE: usize = get_minimal_buffer_size(&RESOLUTION);

    Graphics::initialize(
        screen_device,
        pointer_device,
        Input_type_type::Pointer,
        BUFFER_SIZE,
        true,
    )
    .await;

    Graphics::get_instance()
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
                Drivers::Std::Console::Standard_in_device_type
            ),
            (
                &"/Devices/Standard_out",
                Drivers::Std::Console::Standard_out_device_type
            ),
            (
                &"/Devices/Standard_error",
                Drivers::Std::Console::Standard_error_device_type
            ),
            (&"/Devices/Time", Drivers::Native::Time_driver_type),
            (&"/Devices/Random", Drivers::Native::Random_device_type),
            (&"/Devices/Null", Drivers::Core::Null_device_type)
        ]
    )
    .await
    .unwrap();

    Mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (&"/Binaries/Command_line_shell", Shell_executable_type),
            (&"/Binaries/File_manager", File_manager_executable_type)
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

    let result = Executable::execute("/Binaries/File_manager", "".to_string(), standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert_eq!(result, 0);
}
