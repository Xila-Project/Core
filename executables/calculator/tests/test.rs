#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[xila::task::run(task_path = xila::task, executor = drivers_std::executor::instantiate_static_executor!())]
async fn run_graphics() {
    use xila::graphics;
    use xila::task;

    let task_manager = task::get_instance();

    const RESOLUTION: graphics::Point = graphics::Point::new(800, 600);
    let (screen_device, pointer_device, keyboard_device, mut runner) =
        drivers_native::window_screen::new(RESOLUTION)
            .await
            .unwrap();

    let task_identifier = task_manager.get_current_task_identifier().await;

    let spawner = task_manager.get_spawner(task_identifier).await.unwrap();

    task_manager
        .spawn(
            task_identifier,
            "Event Loop",
            Some(spawner),
            async move |_| {
                runner.run().await;
            },
        )
        .await
        .unwrap();

    // - - Initialize the graphics manager
    let graphics_manager = graphics::initialize(
        screen_device,
        pointer_device,
        graphics::InputKind::Pointer,
        graphics::get_minimal_buffer_size(&RESOLUTION),
        true,
    )
    .await;

    graphics_manager
        .add_input_device(keyboard_device, graphics::InputKind::Keypad)
        .await
        .unwrap();

    graphics::get_instance()
        .r#loop(task::Manager::sleep)
        .await
        .unwrap();
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[xila::task::test(task_path = xila::task)]
#[ignore]
async fn main() {
    drivers_std::memory::instantiate_global_allocator!();

    extern crate alloc;

    use drivers_native::TimeDriver;
    use std::fs;
    use xila::executable::build_crate;
    use xila::file_system;
    use xila::file_system::{MemoryDevice, create_device, create_file_system};
    use xila::graphics;
    use xila::host_bindings;
    use xila::little_fs;
    use xila::log;
    use xila::task;
    use xila::time;
    use xila::time::Duration;
    use xila::users;
    use xila::virtual_file_system;
    use xila::virtual_file_system::{create_default_hierarchy, mount_static_devices};
    use xila::virtual_machine;

    // - Initialize the system
    log::initialize(&drivers_std::log::Logger).unwrap();

    let wasm_crate_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("./wasm");
    let binary_path = build_crate(&wasm_crate_path).unwrap();
    let binary_buffer = fs::read(&binary_path).unwrap();

    users::initialize();

    let task_instance = task::initialize();

    let task = task_instance.get_current_task_identifier().await;

    time::initialize(create_device!(TimeDriver::new())).expect("Error initializing time manager");

    std::thread::spawn(run_graphics);

    // Wait for graphics manager to be initialized
    while graphics::try_get_instance().is_none() {
        task::Manager::sleep(Duration::from_millis(10)).await;
    }

    let memory_device = create_device!(MemoryDevice::<512>::new(1024 * 512));
    little_fs::FileSystem::format(memory_device.clone(), 512).unwrap();

    let virtual_file_system = virtual_file_system::initialize(
        create_file_system!(little_fs::FileSystem::new(memory_device, 256).unwrap()),
        None,
    )
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

    let virtual_machine = virtual_machine::initialize(&[&host_bindings::GraphicsBindings]);

    let task = task_instance.get_current_task_identifier().await;

    let standard_in = virtual_file_system
        .open(
            &"/devices/standard_in",
            file_system::Mode::READ_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(
            &"/devices/standard_out",
            file_system::Mode::WRITE_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(
            &"/devices/standard_out",
            file_system::Mode::WRITE_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    virtual_machine
        .execute(
            binary_buffer.to_vec(),
            8 * 1024,
            standard_in,
            standard_out,
            standard_error,
        )
        .await
        .unwrap();

    loop {
        task::Manager::sleep(Duration::from_millis(1000)).await;
    }
}
