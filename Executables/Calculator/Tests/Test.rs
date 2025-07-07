#![allow(non_camel_case_types)]

extern crate alloc;

use Xila::Drivers;
use Xila::Drivers::Native::Time_driver_type;
use Xila::File_system;
use Xila::File_system::{Create_device, Create_file_system, Memory_device_type};
use Xila::Graphics;
use Xila::Graphics::LVGL;
use Xila::Host_bindings;
use Xila::LittleFS;
use Xila::Log;
use Xila::Memory::Instantiate_global_allocator;
use Xila::Task;
use Xila::Task::Test;
use Xila::Time;
use Xila::Time::Duration_type;
use Xila::Users;
use Xila::Virtual_file_system;
use Xila::Virtual_file_system::{Create_default_hierarchy, Mount_static_devices};
use Xila::Virtual_machine;

Instantiate_global_allocator!(Drivers::Std::Memory::Memory_manager_type);

#[Task::Run(Executor = Drivers::Std::Executor::Instantiate_static_executor!())]
async fn run_graphics() {
    Graphics::Get_instance()
        .Loop(Task::Manager_type::Sleep)
        .await
        .unwrap();
}

#[ignore]
#[Test]
async fn Integration_test() {
    // - Initialize the system
    Log::Initialize(&Drivers::Std::Log::Logger_type).unwrap();

    let Binary_buffer = include_bytes!("../WASM/target/wasm32-wasip1/release/Calculator.wasm");

    Users::Initialize();

    let Task_instance = Task::Initialize();

    let Task = Task_instance.Get_current_task_identifier().await;

    Time::Initialize(Create_device!(Time_driver_type::new()))
        .expect("Error initializing time manager");

    let Memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));
    LittleFS::File_system_type::Format(Memory_device.clone(), 512).unwrap();

    let Virtual_file_system = Virtual_file_system::Initialize(
        Create_file_system!(LittleFS::File_system_type::new(Memory_device, 256).unwrap()),
        None,
    )
    .unwrap();

    Create_default_hierarchy(Virtual_file_system, Task)
        .await
        .unwrap();

    Mount_static_devices!(
        Virtual_file_system,
        Task,
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

    let Virtual_machine = Virtual_machine::Get_instance();
    const RESOLUTION: Graphics::Point_type = Graphics::Point_type::new(800, 600);
    let (Screen_device, Pointer_device, Keyboard_device) =
        Drivers::Native::Window_screen::New(RESOLUTION).unwrap();
    // - - Initialize the graphics manager
    Graphics::Initialize(
        Screen_device,
        Pointer_device,
        Graphics::Input_type_type::Pointer,
        Graphics::Get_minimal_buffer_size(&RESOLUTION),
        true,
    )
    .await;

    Graphics::Get_instance()
        .Add_input_device(Keyboard_device, Graphics::Input_type_type::Keypad)
        .await
        .unwrap();

    std::thread::spawn(run_graphics);

    let Task = Task_instance.Get_current_task_identifier().await;

    let Graphics_manager = Graphics::Get_instance();

    let Window = Graphics_manager.Create_window().await.unwrap();

    let _Calendar = unsafe { LVGL::lv_calendar_create(Window.Into_raw()) };

    let Standard_in = Virtual_file_system
        .Open(
            &"/Devices/Standard_in",
            File_system::Mode_type::READ_ONLY.into(),
            Task,
        )
        .await
        .unwrap();

    let Standard_out = Virtual_file_system
        .Open(
            &"/Devices/Standard_out",
            File_system::Mode_type::WRITE_ONLY.into(),
            Task,
        )
        .await
        .unwrap();

    let Standard_error = Virtual_file_system
        .Open(
            &"/Devices/Standard_out",
            File_system::Mode_type::WRITE_ONLY.into(),
            Task,
        )
        .await
        .unwrap();

    Virtual_machine
        .Execute(
            Binary_buffer.to_vec(),
            8 * 1024,
            Standard_in,
            Standard_out,
            Standard_error,
        )
        .await
        .unwrap();

    loop {
        Task::Manager_type::Sleep(Duration_type::from_millis(1000)).await;
    }
}
