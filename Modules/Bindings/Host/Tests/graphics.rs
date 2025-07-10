extern crate alloc;

use drivers::native::TimeDriverType;
use file_system::{create_device, Create_file_system, MemoryDeviceType};
use graphics::lvgl;
use memory::Instantiate_global_allocator;
use task::test;
use time::Duration;
use virtual_file_system::{create_default_hierarchy, Mount_static_devices};

Instantiate_global_allocator!(drivers::standard_library::memory::MemoryManager);

#[task::Run(executor = drivers::standard_library::executor::instantiate_static_executor!())]
async fn run_graphics() {
    graphics::get_instance()
        .r#loop(task::Manager::sleep)
        .await
        .unwrap();
}

#[ignore]
#[test]
async fn i() {
    // - Initialize the system
    log::initialize(&drivers::standard_library::log::LoggerType).unwrap();

    let binary_buffer = include_bytes!("./WASM_test/target/wasm32-wasip1/release/WASM_test.wasm");

    users::Initialize();

    let task_instance = task::initialize();

    let task = task_instance.get_current_task_identifier().await;

    time::initialize(create_device!(TimeDriverType::new()))
        .expect("Error initializing time manager");

    let memory_device = create_device!(MemoryDeviceType::<512>::new(1024 * 512));
    little_fs::FileSystem::format(memory_device.clone(), 512).unwrap();

    let virtual_file_system = virtual_file_system::initialize(
        Create_file_system!(little_fs::FileSystemType::new(memory_device, 256).unwrap()),
        None,
    )
    .unwrap();

    create_default_hierarchy(virtual_file_system, task)
        .await
        .unwrap();

    Mount_static_devices!(
        virtual_file_system,
        task,
        &[
            (
                &"/Devices/Standard_in",
                drivers::standard_library::console::StandardInDevice
            ),
            (
                &"/Devices/Standard_out",
                drivers::standard_library::console::StandardOutDeviceType
            ),
            (
                &"/Devices/Standard_error",
                drivers::standard_library::console::StandardErrorDeviceType
            ),
            (&"/Devices/Time", drivers::native::TimeDriverType),
            (&"/Devices/Random", drivers::native::RandomDeviceType),
            (&"/Devices/Null", drivers::core::NullDeviceType)
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
            &"/Devices/Standard_in",
            file_system::Mode::READ_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(
            &"/Devices/Standard_out",
            file_system::Mode::WRITE_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(
            &"/Devices/Standard_out",
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
