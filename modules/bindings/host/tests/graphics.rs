extern crate abi_definitions;
extern crate alloc;
extern crate std;

use drivers_native::TimeDriver;
use executable::{Standard, build_crate};
use file_system::{MemoryDevice, create_device, create_file_system};
use graphics::lvgl;
use std::fs;
use task::test;
use time::Duration;
use virtual_file_system::{create_default_hierarchy, mount_static_devices};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
drivers_std::memory::instantiate_global_allocator!();

#[task::run(executor = drivers_std::executor::instantiate_static_executor!())]
async fn run_graphics() {
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

#[ignore]
#[test]
async fn test() {
    // - Initialize the system
    log::initialize(&drivers_std::log::Logger).unwrap();

    let binary_path = build_crate(&"host_bindings_wasm_test").unwrap();
    let binary_buffer = fs::read(&binary_path).unwrap();

    users::initialize();

    let task_instance = task::initialize();

    let task = task_instance.get_current_task_identifier().await;

    time::initialize(create_device!(TimeDriver::new())).expect("Error initializing time manager");

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

    virtual_machine::initialize(&[&host_bindings::GraphicsBindings]);

    let virtual_machine = virtual_machine::get_instance();

    std::thread::spawn(run_graphics);

    // Wait for graphics manager to be initialized
    while graphics::try_get_instance().is_none() {
        task::Manager::sleep(Duration::from_millis(10)).await;
    }
    let graphics_manager = graphics::get_instance();

    let task = task_instance.get_current_task_identifier().await;

    let window = graphics_manager.create_window().await.unwrap();

    let _calendar = unsafe { lvgl::lv_calendar_create(window.into_raw()) };

    let standard = Standard::open(
        &"/devices/standard_in",
        &"/devices/standard_out",
        &"/devices/standard_error",
        task,
        virtual_file_system,
    )
    .await
    .unwrap()
    .split();

    virtual_machine
        .execute(
            binary_buffer.to_vec(),
            8 * 1024,
            standard,
            None,
            vec![],
            task,
        )
        .await
        .unwrap();

    loop {
        task::Manager::sleep(Duration::from_millis(1000)).await;
    }
}
