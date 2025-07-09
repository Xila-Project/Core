#![allow(non_camel_case_types)]

extern crate alloc;

use drivers::native::Time_driver_type;
use file_system::{Create_device, Create_file_system, Memory_device_type};
use graphics::lvgl;
use memory::Instantiate_global_allocator;
use task::Test;
use time::Duration_type;
use virtual_file_system::{create_default_hierarchy, Mount_static_devices};

Instantiate_global_allocator!(drivers::standard_library::memory:
:Memory_manager_type);

#[task::Run(executor = drivers::standard_library::executor::Instantiate_static_executor!())]
async fn run_graphics() {
    graphics::get_instance()
        .r#loop(task::Manager_type::Sleep)
        .await
        .unwrap();
}

#[ignore]
#[Test]
async fn i() {
    // - Initialize the system
    log::initialize(&drivers::standard_library::log::Logger_type).unwrap();

    let binary_buffer = include_bytes!("./WASM_test/target/wasm32-wasip1/release/WASM_test.wasm");

    users::Initialize();

    let task_instance = task::initialize();

    let task = task_instance.get_current_task_identifier().await;

    time::initialize(Create_device!(Time_driver_type::new()))
        .expect("Error initializing time manager");

    let memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));
    little_fs::File_system_type::format(memory_device.clone(), 512).unwrap();

    let virtual_file_system = virtual_file_system::initialize(
        Create_file_system!(little_fs::File_system_type::new(memory_device, 256).unwrap()),
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

    virtual_machine::Initialize(&[&host_bindings::Graphics_bindings]);

    let virtual_machine = virtual_machine::get_instance();
    const RESOLUTION: graphics::Point_type = graphics::Point_type::new(800, 600);
    let (screen_device, pointer_device, keyboard_device) =
        drivers::native::window_screen::new(RESOLUTION).unwrap();
    // - - Initialize the graphics manager
    graphics::initialize(
        screen_device,
        pointer_device,
        graphics::Input_type_type::Pointer,
        graphics::get_minimal_buffer_size(&RESOLUTION),
        true,
    )
    .await;

    graphics::get_instance()
        .add_input_device(keyboard_device, graphics::Input_type_type::Keypad)
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
            file_system::Mode_type::READ_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(
            &"/Devices/Standard_out",
            file_system::Mode_type::WRITE_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(
            &"/Devices/Standard_out",
            file_system::Mode_type::WRITE_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    virtual_machine
        .Execute(
            binary_buffer.to_vec(),
            8 * 1024,
            standard_in,
            standard_out,
            standard_error,
        )
        .await
        .unwrap();

    loop {
        task::Manager_type::Sleep(Duration_type::from_millis(1000)).await;
    }
}
