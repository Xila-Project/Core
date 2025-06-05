#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

use Executable::Standard_type;
use File_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use Task::Test;
use Terminal::Terminal_executable_type;

#[cfg(target_os = "linux")]
#[ignore]
#[Test]
async fn main() {
    use alloc::string::ToString;
    use Command_line_shell::Shell_executable_type;
    use Drivers::Native::Window_screen;
    use Graphics::{Get_minimal_buffer_size, Input_type_type, Point_type};

    // - Initialize the task manager.
    let Task_instance = Task::Initialize();

    // - Initialize the user manager.
    let _ = Users::Initialize();

    // - Initialize the time manager.
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    // - Initialize the graphics manager.

    const Resolution: Point_type = Point_type::New(800, 480);

    let (Screen_device, Pointer_device, Keyboard_device) = Window_screen::New(Resolution).unwrap();

    const Buffer_size: usize = Get_minimal_buffer_size(&Resolution);

    Graphics::Initialize(
        Screen_device,
        Pointer_device,
        Input_type_type::Pointer,
        Buffer_size,
        true,
    )
    .await;

    Graphics::Get_instance()
        .Add_input_device(Keyboard_device, Input_type_type::Keypad)
        .await
        .unwrap();

    // - Initialize the virtual file system.
    let Memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::Format(Memory_device.clone(), 256).unwrap();

    let File_system = LittleFS::File_system_type::New(Memory_device, 256).unwrap();

    Virtual_file_system::Initialize(Create_file_system!(File_system), None).unwrap();

    let Task = Task_instance.Get_current_task_identifier().await;

    Virtual_file_system::Get_instance()
        .Create_directory(&"/Binaries", Task)
        .await
        .unwrap();

    Virtual_file_system::Get_instance()
        .Mount_static_device(
            Task,
            &"/Binaries/Command_line_shell",
            Create_device!(Shell_executable_type),
        )
        .await
        .unwrap();

    Virtual_file_system::Get_instance()
        .Mount_static_device(Task, &"/Terminal", Create_device!(Terminal_executable_type))
        .await
        .unwrap();

    Virtual_file_system::Get_instance()
        .Create_directory(&"/Devices", Task)
        .await
        .unwrap();

    Drivers::Native::Console::Mount_devices(Task, Virtual_file_system::Get_instance())
        .await
        .unwrap();

    let Standard_in = Virtual_file_system::Get_instance()
        .Open(&"/Devices/Standard_in", Mode_type::Read_only.into(), Task)
        .await
        .unwrap();

    let Standard_out = Virtual_file_system::Get_instance()
        .Open(&"/Devices/Standard_out", Mode_type::Write_only.into(), Task)
        .await
        .unwrap();

    let Standard_error = Virtual_file_system::Get_instance()
        .Open(
            &"/Devices/Standard_error",
            Mode_type::Write_only.into(),
            Task,
        )
        .await
        .unwrap();

    let Standard = Standard_type::New(
        Standard_in,
        Standard_out,
        Standard_error,
        Task,
        Virtual_file_system::Get_instance(),
    );

    Task_instance
        .Set_environment_variable(Task, "Paths", "/")
        .await
        .unwrap();

    Task_instance
        .Set_environment_variable(Task, "Host", "xila")
        .await
        .unwrap();

    let Result = Executable::Execute("/Terminal", "".to_string(), Standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert_eq!(Result, 0);
}
