extern crate alloc;

use executable::Standard;
use file_system::{create_device, create_file_system, MemoryDevice, Mode};
use graphical_shell::ShellExecutableType;
use task::test;

#[cfg(target_os = "linux")]
#[ignore]
#[test]
async fn main() {
    use alloc::string::ToString;
    use drivers::native::window_screen;
    use executable::mount_static_executables;
    use file_system::{Flags, Open};
    use graphics::{get_minimal_buffer_size, InputKind, Point};
    use users::GroupIdentifier;

    use virtual_file_system::{File, Mount_static_devices};

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
    let memory_device = create_device!(MemoryDevice::<512>::new(1024 * 512));

    little_fs::FileSystem::format(memory_device.clone(), 256).unwrap();

    let file_system = little_fs::FileSystem::new(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(create_file_system!(file_system), None).unwrap();

    let task = task_instance.get_current_task_identifier().await;

    virtual_file_system::create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    mount_static_executables!(
        virtual_file_system,
        task,
        &[(&"/binaries/Graphical_shell", ShellExecutableType),]
    )
    .await
    .unwrap();

    Mount_static_devices!(
        virtual_file_system,
        task,
        &[
            (
                &"/devices/Standard_in",
                drivers::standard_library::console::StandardInDevice
            ),
            (
                &"/devices/Standard_out",
                drivers::standard_library::console::StandardOutDeviceType
            ),
            (
                &"/devices/Standard_error",
                drivers::standard_library::console::StandardErrorDeviceType
            ),
            (&"/devices/Time", drivers::native::TimeDriverType),
            (&"/devices/Random", drivers::native::RandomDeviceType),
            (&"/devices/Null", drivers::core::NullDeviceType)
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

        File::open(
            virtual_file_system,
            format!("/Configuration/Shared/Shortcuts/Test{i}.json").as_str(),
            Flags::new(Mode::WRITE_ONLY, Some(Open::CREATE), None),
        )
        .await
        .unwrap()
        .write(
            format!(
                r#"
    {{
        "name": "Test{i}",
        "command": "/binaries/?",
        "arguments": "",
        "terminal": false,
        "icon_string": "T!",
        "icon_color": [255, 0, 0]
    }}
        "#
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    }

    let group_identifier = GroupIdentifier::new(1000);

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

    let standard = Standard::open(
        &"/devices/Standard_in",
        &"/devices/Standard_out",
        &"/devices/Standard_error",
        task,
        virtual_file_system,
    )
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

    let result = executable::execute("/binaries/Graphical_shell", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
