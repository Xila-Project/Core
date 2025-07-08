#![no_std]
#![allow(non_camel_case_types)]

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
    let task_instance = Task::Initialize();

    // - Initialize the user manager.
    let _ = Users::Initialize();

    // - Initialize the time manager.
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::new()));

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

    // - Initialize the virtual file system.
    let memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::format(memory_device.clone(), 256).unwrap();

    let file_system = LittleFS::File_system_type::new(memory_device, 256).unwrap();

    let virtual_file_system =
        Virtual_file_system::initialize(Create_file_system!(file_system), None).unwrap();

    let task = task_instance.get_current_task_identifier().await;

    Virtual_file_system::create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    Mount_static_executables!(
        virtual_file_system,
        task,
        &[(&"/Binaries/Graphical_shell", Shell_executable_type),]
    )
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

    virtual_file_system
        .create_directory(&"/Configuration/Shared/Shortcuts", task)
        .await
        .unwrap();

    // Add fake shortcuts.
    for i in 0..20 {
        use alloc::format;

        File_type::open(
            virtual_file_system,
            format!("/Configuration/Shared/Shortcuts/Test{i}.json").as_str(),
            Flags_type::New(Mode_type::WRITE_ONLY, Some(Open_type::CREATE), None),
        )
        .await
        .unwrap()
        .write(
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

    let group_identifier = Group_identifier_type::New(1000);

    Authentication::create_group(virtual_file_system, "alix_anneraud", Some(group_identifier))
        .await
        .unwrap();

    Authentication::create_user(
        virtual_file_system,
        "alix_anneraud",
        "password",
        group_identifier,
        None,
    )
    .await
    .unwrap();

    let standard = Standard_type::open(
        &"/Devices/Standard_in",
        &"/Devices/Standard_out",
        &"/Devices/Standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap();

    task_instance
        .Set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_instance
        .Set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();

    let result = Executable::execute("/Binaries/Graphical_shell", "".to_string(), standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert!(result == 0);
}
