#![allow(non_camel_case_types)]

extern crate alloc;

use drivers::Native::Time_driver_type;
use file_system::{Create_device, Create_file_system, Memory_device_type};
use graphics::LVGL;
use memory::Instantiate_global_allocator;
use task::Test;
use time::Duration_type;
use virtual_file_system::{create_default_hierarchy, Mount_static_devices};

Instantiate_global_allocator!(Drivers::Std::Memory::Memory_manager_type);

#[Task::Run(executor = Drivers::Std::Executor::Instantiate_static_executor!())]
async fn run_graphics() {
    Graphics::get_instance()
        .r#loop(Task::Manager_type::Sleep)
        .await
        .unwrap();
}

#[ignore]
#[Test]
async fn i() {
    // - Initialize the system
    Log::Initialize(&Drivers::Std::Log::Logger_type).unwrap();

    let binary_buffer = include_bytes!("./WASM_test/target/wasm32-wasip1/release/WASM_test.wasm");

    Users::Initialize();

    let task_instance = Task::Initialize();

    let task = task_instance.get_current_task_identifier().await;

    Time::Initialize(Create_device!(Time_driver_type::new()))
        .expect("Error initializing time manager");

    let memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));
    LittleFS::File_system_type::format(memory_device.clone(), 512).unwrap();

    let virtual_file_system = Virtual_file_system::initialize(
        Create_file_system!(LittleFS::File_system_type::new(memory_device, 256).unwrap()),
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
                Drivers::Std::Console::Standard_in_device_type
            ),
            (
                &"/Devices/Standard_out",
                Drivers::Std::Console::Standard_out_device_type
            ),
            (
                &"/Devices/Standard_error",
                Drivers::Std::Console::Standard_error_device_type
            ),
            (&"/Devices/Time", Drivers::Native::Time_driver_type),
            (&"/Devices/Random", Drivers::Native::Random_device_type),
            (&"/Devices/Null", Drivers::Core::Null_device_type)
        ]
    )
    .await
    .unwrap();

    Virtual_machine::Initialize(&[&Host_bindings::Graphics_bindings]);

    let virtual_machine = Virtual_machine::get_instance();
    const RESOLUTION: Graphics::Point_type = Graphics::Point_type::new(800, 600);
    let (screen_device, pointer_device, keyboard_device) =
        Drivers::Native::Window_screen::New(RESOLUTION).unwrap();
    // - - Initialize the graphics manager
    Graphics::initialize(
        screen_device,
        pointer_device,
        Graphics::Input_type_type::Pointer,
        Graphics::get_minimal_buffer_size(&RESOLUTION),
        true,
    )
    .await;

    Graphics::get_instance()
        .add_input_device(keyboard_device, Graphics::Input_type_type::Keypad)
        .await
        .unwrap();

    std::thread::spawn(run_graphics);

    let task = task_instance.get_current_task_identifier().await;

    let graphics_manager = Graphics::get_instance();

    let window = graphics_manager.create_window().await.unwrap();

    let _calendar = unsafe { LVGL::lv_calendar_create(window.into_raw()) };

    let standard_in = virtual_file_system
        .open(
            &"/Devices/Standard_in",
            File_system::Mode_type::READ_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard_out = virtual_file_system
        .open(
            &"/Devices/Standard_out",
            File_system::Mode_type::WRITE_ONLY.into(),
            task,
        )
        .await
        .unwrap();

    let standard_error = virtual_file_system
        .open(
            &"/Devices/Standard_out",
            File_system::Mode_type::WRITE_ONLY.into(),
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
        Task::Manager_type::Sleep(Duration_type::from_millis(1000)).await;
    }
}
