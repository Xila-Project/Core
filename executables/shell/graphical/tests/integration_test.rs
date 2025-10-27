#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;
    extern crate abi_definitions;

    use drivers_native::window_screen;
    use graphical_shell::ShellExecutable;
    use xila::executable::Standard;
    use xila::executable::mount_static_executables;
    use xila::file_system::{Flags, MemoryDevice, Mode, Open, create_device, create_file_system};
    use xila::graphics::{self, InputKind, Point, get_minimal_buffer_size};
    use xila::users::GroupIdentifier;
    use xila::virtual_file_system::{File, mount_static_devices};
    use xila::{authentication, executable, task, time, users, virtual_file_system};

    // - Initialize the task manager.
    let task_manager = task::initialize();

    let task = task_manager.get_current_task_identifier().await;

    // - Initialize the user manager.
    let _ = users::initialize();

    // - Initialize the time manager.
    let _ = time::initialize(create_device!(drivers_native::TimeDriver::new()));

    // - Initialize the graphics manager.

    const RESOLUTION: Point = Point::new(800, 480);

    let (screen_device, pointer_device, keyboard_device, mut runner) =
        window_screen::new(RESOLUTION).await.unwrap();

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

    task_manager
        .spawn(task, "Window screen runner", None, async move |_| {
            runner.run().await;
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
                drivers_std::console::StandardInDevice
            ),
            (
                &"/devices/standard_out",
                drivers_std::console::StandardOutDevice
            ),
            (
                &"/devices/standard_error",
                drivers_std::console::StandardErrorDevice
            ),
            (&"/devices/time", drivers_native::TimeDriver),
            (&"/devices/random", drivers_shared::devices::RandomDevice),
            (&"/devices/null", drivers_core::NullDevice)
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
        "arguments": ["test"],
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

    let result = executable::execute("/binaries/graphical_shell", vec![], standard, None)
        .await
        .unwrap()
        .join()
        .await;

    assert!(result == 0);
}
