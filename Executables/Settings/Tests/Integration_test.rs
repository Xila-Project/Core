#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;

use Executable::Standard_type;
use File_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use Settings::Settings_executable_type;
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
    use Virtual_file_system::{Create_default_hierarchy, Mount_static_devices};

    // - Initialize the task manager.
    let Task_instance = Task::Initialize();

    // - Initialize the user manager.
    let _ = Users::Initialize();

    // - Initialize the time manager.
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    // - Initialize the virtual file system.
    let Memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::Format(Memory_device.clone(), 256).unwrap();

    let File_system = LittleFS::File_system_type::New(Memory_device, 256).unwrap();

    let Virtual_file_system =
        Virtual_file_system::Initialize(Create_file_system!(File_system), None).unwrap();

    // - Initialize the graphics manager.

    const RESOLUTION: Point_type = Point_type::New(800, 480);

    let (Screen_device, Pointer_device, Keyboard_device) = Window_screen::New(RESOLUTION).unwrap();

    const BUFFER_SIZE: usize = Get_minimal_buffer_size(&RESOLUTION);

    Graphics::Initialize(
        Screen_device,
        Pointer_device,
        Input_type_type::Pointer,
        BUFFER_SIZE,
        true,
    )
    .await;

    Graphics::Get_instance()
        .Add_input_device(Keyboard_device, Input_type_type::Keypad)
        .await
        .unwrap();

    let Task = Task_instance.Get_current_task_identifier().await;

    Create_default_hierarchy(Virtual_file_system, Task)
        .await
        .unwrap();

    Mount_static_devices!(
        Virtual_file_system,
        Task,
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
        Virtual_file_system,
        Task,
        &[
            (&"/Binaries/Command_line_shell", Shell_executable_type),
            (&"/Binaries/Settings", Settings_executable_type)
        ]
    )
    .await
    .unwrap();

    let Standard_in = Virtual_file_system
        .Open(&"/Devices/Standard_in", Mode_type::READ_ONLY.into(), Task)
        .await
        .unwrap();

    let Standard_out = Virtual_file_system
        .Open(&"/Devices/Standard_out", Mode_type::WRITE_ONLY.into(), Task)
        .await
        .unwrap();

    let Standard_error = Virtual_file_system
        .Open(
            &"/Devices/Standard_error",
            Mode_type::WRITE_ONLY.into(),
            Task,
        )
        .await
        .unwrap();

    let Standard = Standard_type::New(
        Standard_in,
        Standard_out,
        Standard_error,
        Task,
        Virtual_file_system,
    );

    Task_instance
        .Set_environment_variable(Task, "Paths", "/")
        .await
        .unwrap();

    Task_instance
        .Set_environment_variable(Task, "Host", "xila")
        .await
        .unwrap();

    let Result = Executable::Execute("/Binaries/Settings", "".to_string(), Standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert_eq!(Result, 0);
}
