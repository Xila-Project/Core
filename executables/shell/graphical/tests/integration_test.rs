extern crate alloc;

use executable::Standard;
use file_system::{MemoryDevice, Mode, create_device, create_file_system};
use graphical_shell::ShellExecutable;
use task::test;

#[cfg(target_os = "linux")]
#[ignore]
#[test]
async fn main() {
    use alloc::string::ToString;
    use drivers::native::window_screen;
    use executable::mount_static_executables;
    use file_system::{Flags, Open};
    use graphics::{InputKind, Point, get_minimal_buffer_size};
    use users::GroupIdentifier;

    use virtual_file_system::{File, mount_static_devices};

    // - Initialize the task manager.
    let task_manager = task::initialize();

    let task = task_manager.get_current_task_identifier().await;

    // - Initialize the user manager.
    let _ = users::initialize();

    // - Initialize the time manager.
    let _ = time::initialize(create_device!(drivers::native::TimeDriver::new()));

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

    // - Initialize the virtual file system.
    let memory_device = create_device!(MemoryDevice::<512>::new(1024 * 512));

    little_fs::FileSystem::format(memory_device.clone(), 256).unwrap();

    let file_system = little_fs::FileSystem::new(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(create_file_system!(file_system), None).unwrap();

    let task = task_manager.get_current_task_identifier().await;

    virtual_file_system::create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    mount_static_executables!(
        virtual_file_system,
        task,
        &[(&"/binaries/graphical_shell", ShellExecutable),]
    )
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
            (&"/devices/time", drivers::native::TimeDriver),
            (&"/devices/random", drivers::native::RandomDevice),
            (&"/devices/null", drivers::core::NullDevice)
        ]
    )
    .await
    .unwrap();

    virtual_file_system
        .create_directory(&"/configuration/shared/shortcuts", task)
        .await
        .unwrap();

    // Add fake shortcuts.
    for i in 0..20 {
        use alloc::format;

        File::open(
            virtual_file_system,
            format!("/configuration/shared/shortcuts/test{i}.json").as_str(),
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
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap();

    task_manager
        .set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_manager
        .set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();

    let result = executable::execute("/binaries/graphical_shell", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
