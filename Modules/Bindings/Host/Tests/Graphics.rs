#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use Drivers::Native::{Time_driver_type, Window_screen};
use File_system::{Create_device, Create_file_system, Tests::Memory_device_type};
use Graphics::{lvgl, Get_recommended_buffer_size, Point_type};
use Time::Duration_type;
#[test]
fn Integration_test() {
    let Binary_buffer = include_bytes!("./WASM_test/target/wasm32-wasip1/release/WASM_test.wasm");

    Users::Initialize().expect("Error initializing users manager");

    let Task_instance = Task::Initialize().expect("Error initializing task manager");

    let Task = Task_instance
        .Get_current_task_identifier()
        .expect("Failed to get current task identifier");

    Time::Initialize(Create_device!(Time_driver_type::New()))
        .expect("Error initializing time manager");

    let Memory_device = Create_device!(Memory_device_type::<512>::New(1024 * 512));
    LittleFS::File_system_type::Format(Memory_device.clone(), 512).unwrap();

    let Virtual_file_system = Virtual_file_system::Initialize(Create_file_system!(
        LittleFS::File_system_type::New(Memory_device, 256).unwrap()
    ))
    .unwrap();

    Virtual_file_system
        .Create_directory(&"/Devices", Task)
        .unwrap();

    Drivers::Native::Console::Mount_devices(Task, Virtual_file_system).unwrap();

    Virtual_machine::Initialize(&[&Host_bindings::Graphics_bindings]);

    let Virtual_machine = Virtual_machine::Get_instance();

    const Resolution: Point_type = Point_type::New(800, 480);

    const Buffer_size: usize = Get_recommended_buffer_size(&Resolution);

    let (Screen_device, Pointer_device) = Window_screen::New(Resolution).unwrap();

    let _Task = Task_instance.Get_current_task_identifier().unwrap();

    Graphics::Initialize();

    let Graphics_manager = Graphics::Get_instance();

    let Display = Graphics_manager
        .Create_display::<Buffer_size>(Screen_device, Pointer_device, false)
        .unwrap();

    let Screen_object = Display.Get_object();

    let _Calendar = unsafe { lvgl::lv_calendar_create(Screen_object) };

    let Standard_in = Virtual_file_system
        .Open(
            &"/Devices/Standard_in",
            File_system::Mode_type::Read_only.into(),
            _Task,
        )
        .unwrap();

    let Standard_out = Virtual_file_system
        .Open(
            &"/Devices/Standard_out",
            File_system::Mode_type::Write_only.into(),
            _Task,
        )
        .unwrap();

    let Standard_error = Virtual_file_system
        .Open(
            &"/Devices/Standard_out",
            File_system::Mode_type::Write_only.into(),
            _Task,
        )
        .unwrap();

    Virtual_machine
        .Instantiate(
            Binary_buffer.to_vec(),
            8 * 1024,
            Standard_in,
            Standard_out,
            Standard_error,
        )
        .unwrap();

    loop {
        Task::Manager_type::Sleep(Duration_type::from_millis(1000));
    }
}
