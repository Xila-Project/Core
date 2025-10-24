#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[ignore]
#[xila::task::test(task_path = xila::task)]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;
    extern crate abi_definitions;

    use alloc::string::ToString;
    use command_line_shell::ShellExecutable;
    use drivers_native::window_screen;
    use file_manager::FileManagerExecutable;
    use xila::executable::Standard;
    use xila::executable::mount_static_executables;
    use xila::file_system::{MemoryDevice, Mode, create_device, create_file_system};
    use xila::graphics::{self, InputKind, Point, get_minimal_buffer_size};
    use xila::virtual_file_system::{self, create_default_hierarchy, mount_static_devices};
    use xila::{executable, task, time, users};

    // - Initialize the task manager.
    let task_manager = task::initialize();

    let task = task_manager.get_current_task_identifier().await;

    // - Initialize the user manager.
    let _ = users::initialize();

    // - Initialize the time manager.
    let _ = time::initialize(create_device!(drivers_native::TimeDriver::new()));

    // - Initialize the virtual file system.
    let memory_device = create_device!(MemoryDevice::<512>::new(1024 * 512));

    little_fs::FileSystem::format(memory_device.clone(), 256).unwrap();

    let file_system = little_fs::FileSystem::new(memory_device, 256).unwrap();

    let virtual_file_system =
        virtual_file_system::initialize(create_file_system!(file_system), None).unwrap();

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

    create_default_hierarchy(virtual_file_system, task)
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

    mount_static_executables!(
        virtual_file_system,
        task,
        &[
            (&"/binaries/command_line_shell", ShellExecutable),
            (&"/binaries/file_manager", FileManagerExecutable)
        ]
    )
    .await
    .unwrap();

    let standard_in = virtual_file_system
        .open(&"/devices/standard_in", Mode::READ_ONLY.into(), task)
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(&"/devices/standard_out", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(&"/devices/standard_error", Mode::WRITE_ONLY.into(), task)
        .await
        .unwrap();

    let standard = Standard::new(
        standard_in,
        standard_out,
        standard_error,
        task,
        virtual_file_system,
    );

    task_manager
        .set_environment_variable(task, "Paths", "/")
        .await
        .unwrap();

    task_manager
        .set_environment_variable(task, "Host", "xila")
        .await
        .unwrap();

    let result = executable::execute("/binaries/file_manager", "".to_string(), standard)
        .await
        .unwrap()
        .join()
        .await;

    assert_eq!(result, 0);
}
