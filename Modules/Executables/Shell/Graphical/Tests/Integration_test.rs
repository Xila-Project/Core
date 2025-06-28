#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

use Executable::Standard_type;
use File_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use Graphical_shell::Shell_executable_type;
use Task::Test;

#[cfg(target_os = "linux")]
#[ignore]
#[Test]
async fn main() {
    use alloc::string::ToString;
    use Drivers::Native::Window_screen;
    use Executable::Mount_static_executables;
    use File_system::{Flags_type, Open_type};
    use Graphics::{Get_minimal_buffer_size, Input_type_type, Point_type};
    use Users::Group_identifier_type;

    use Virtual_file_system::{File_type, Mount_static_devices};

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

    let Virtual_file_system =
        Virtual_file_system::Initialize(Create_file_system!(File_system), None).unwrap();

    let Task = Task_instance.Get_current_task_identifier().await;

    Virtual_file_system::Create_default_hierarchy(Virtual_file_system, Task)
        .await
        .unwrap();

    Mount_static_executables!(
        Virtual_file_system,
        Task,
        &[(&"/Binaries/Graphical_shell", Shell_executable_type),]
    )
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

    Virtual_file_system
        .Create_directory(&"/Configuration/Shared/Shortcuts", Task)
        .await
        .unwrap();

    // Add fake shortcuts.
    for i in 0..20 {
        use alloc::format;

        File_type::Open(
            Virtual_file_system,
            format!("/Configuration/Shared/Shortcuts/Test{i}.json").as_str(),
            Flags_type::New(Mode_type::Write_only, Some(Open_type::Create), None),
        )
        .await
        .unwrap()
        .Write(
            format!(
                r#"
    {{
        "Name": "Test{i}",
        "Command": "/Binaries/?",
        "Arguments": "",
        "Terminal": false,
        "Icon_string": "T!",
        "Icon_color": [255, 0, 0]
    }}
        "#
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    }

    let Group_identifier = Group_identifier_type::New(1000);

    Authentication::Create_group(Virtual_file_system, "alix_anneraud", Some(Group_identifier))
        .await
        .unwrap();

    Authentication::Create_user(
        Virtual_file_system,
        "alix_anneraud",
        "password",
        Group_identifier,
        None,
    )
    .await
    .unwrap();

    let Standard = Standard_type::Open(
        &"/Devices/Standard_in",
        &"/Devices/Standard_out",
        &"/Devices/Standard_error",
        Task,
        Virtual_file_system,
    )
    .await
    .unwrap();

    Task_instance
        .Set_environment_variable(Task, "Paths", "/")
        .await
        .unwrap();

    Task_instance
        .Set_environment_variable(Task, "Host", "xila")
        .await
        .unwrap();

    let Result = Executable::Execute("/Binaries/Graphical_shell", "".to_string(), Standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert!(Result == 0);
}
