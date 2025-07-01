#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate alloc;

use Drivers::Native::Time_driver_type;
use File_system::{Create_device, Create_file_system, Memory_device_type};
use Graphics::LVGL;
use Memory::Instantiate_global_allocator;
use Task::Test;
use Time::Duration_type;
use Virtual_file_system::{Create_default_hierarchy, Mount_static_devices};

Instantiate_global_allocator!(Drivers::Std::Memory::Memory_manager_type);

#[ignore]
#[Test]
async fn Integration_test() {
    // - Initialize the system
    Log::Initialize(&Drivers::Std::Log::Logger_type).unwrap();

    let Binary_buffer = include_bytes!("./WASM_test/target/wasm32-wasip1/release/WASM_test.wasm");

    Users::Initialize();

    let Task_instance = Task::Initialize();

    let Task = Task_instance.Get_current_task_identifier().await;

    Time::Initialize(Create_device!(Time_driver_type::New()))
        .expect("Error initializing time manager");

    let Memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));
    LittleFS::File_system_type::Format(Memory_device.clone(), 512).unwrap();

    let Virtual_file_system = Virtual_file_system::Initialize(
        Create_file_system!(LittleFS::File_system_type::New(Memory_device, 256).unwrap()),
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
    const Resolution: Graphics::Point_type = Graphics::Point_type::New(800, 600);
    let (Screen_device, Pointer_device, Keyboard_device) =
        Drivers::Native::Window_screen::New(Resolution).unwrap();
    // - - Initialize the graphics manager
    Graphics::Initialize(
        Screen_device,
        Pointer_device,
        Graphics::Input_type_type::Pointer,
        Graphics::Get_minimal_buffer_size(&Resolution),
        true,
    )
    .await;

    Graphics::Get_instance()
        .Add_input_device(Keyboard_device, Graphics::Input_type_type::Keypad)
        .await
        .unwrap();

    let Task = Task_instance.Get_current_task_identifier().await;

    let Graphics_manager = Graphics::Get_instance();

    let Window = Graphics_manager.Create_window().await.unwrap();

    let _Calendar = unsafe { LVGL::lv_calendar_create(Window.Into_raw()) };

    let Standard_in = Virtual_file_system
        .Open(
            &"/Devices/Standard_in",
            File_system::Mode_type::Read_only.into(),
            Task,
        )
        .await
        .unwrap();

    let Standard_out = Virtual_file_system
        .Open(
            &"/Devices/Standard_out",
            File_system::Mode_type::Write_only.into(),
            Task,
        )
        .await
        .unwrap();

    let Standard_error = Virtual_file_system
        .Open(
            &"/Devices/Standard_out",
            File_system::Mode_type::Write_only.into(),
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
