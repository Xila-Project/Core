extern crate alloc;
extern crate std;

use drivers::native::TimeDriver;
use executable::build_crate;
use file_system::{MemoryDevice, create_device, create_file_system};
use graphics::lvgl;
use memory::instantiate_global_allocator;
use std::fs;
use task::test;
use time::Duration;
use virtual_file_system::{create_default_hierarchy, mount_static_devices};

instantiate_global_allocator!(drivers::standard_library::memory::MemoryManager);

#[task::run(executor = drivers::standard_library::executor::instantiate_static_executor!())]
async fn run_graphics() {
    graphics::get_instance()
        .r#loop(task::Manager::sleep)
        .await
        .unwrap();
}

#[ignore]
#[test]
async fn test() {
    // - Initialize the system
    log::initialize(&drivers::standard_library::log::Logger).unwrap();

    let wasm_crate_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("./tests/wasm_test");
    let binary_path = build_crate(&wasm_crate_path).unwrap();
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
            (&"/devices/random", drivers::shared::devices::RandomDevice),
            (&"/devices/null", drivers::core::NullDevice)
        ]
    )
    .await
    .unwrap();

    virtual_machine::initialize(&[&host_bindings::GraphicsBindings]);

    let virtual_machine = virtual_machine::get_instance();
    const RESOLUTION: graphics::Point = graphics::Point::new(800, 600);
    let (screen_device, pointer_device, keyboard_device) =
        drivers::native::window_screen::new(RESOLUTION).unwrap();
    // - - Initialize the graphics manager
    graphics::initialize(
        screen_device,
        pointer_device,
        graphics::InputKind::Pointer,
        graphics::get_minimal_buffer_size(&RESOLUTION),
        true,
    )
    .await;

    graphics::get_instance()
        .add_input_device(keyboard_device, graphics::InputKind::Keypad)
        .await
        .unwrap();

    std::thread::spawn(run_graphics);

    let task = task_instance.get_current_task_identifier().await;

    let graphics_manager = graphics::get_instance();

    let window = graphics_manager.create_window().await.unwrap();

    let _calendar = unsafe { lvgl::lv_calendar_create(window.into_raw()) };

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
