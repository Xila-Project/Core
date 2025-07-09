#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use executable::Standard_type;
use file_system::{Create_device, Create_file_system, Memory_device_type, Mode_type};
use graphical_shell::Shell_executable_type;
use task::Test;

#[cfg(target_os = "linux")]
#[ignore]
#[Test]
async fn main() {
    use alloc::string::ToString;
    use drivers::native::window_screen;
    use executable::Mount_static_executables;
    use file_system::{Flags_type, Open_type};
    use graphics::{Get_minimal_buffer_size, Input_type_type, Point_type};
    use users::Group_identifier_type;

    use virtual_file_system::{File_type, Mount_static_devices};

    // - Initialize the task manager.
    let task_instance = task::Initialize();

    // - Initialize the user manager.
    let _ = users::Initialize();

    // - Initialize the time manager.
    let _ = time::Initialize(Create_device!(drivers::native::Time_driver_type::new()));

    // - Initialize the graphics manager.

    const RESOLUTION: Point_type = Point_type::new(800, 480);

    let (screen_device, pointer_device, keyboard_device) = window_screen::New(RESOLUTION).unwrap();

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

    // - Initialize the virtual file system.
    let memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    little_fs::File_system_type::format(memory_device.clone(), 256).unwrap();

    let file_system = little_fs::File_system_type::new(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(Create_file_system!(file_system), None).unwrap();

    let task = task_instance.get_current_task_identifier().await;

    virtual_file_system::create_default_hierarchy(virtual_file_system, task)
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

    authentication::create_group(virtual_file_system, "alix_anneraud", Some(group_identifier))
        .await
        .unwrap();

    authentication::create_user(
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

    let result = executable::execute("/Binaries/Graphical_shell", "".to_string(), standard)
        .await
        .unwrap()
        .Join()
        .await;

    assert!(result == 0);
}
