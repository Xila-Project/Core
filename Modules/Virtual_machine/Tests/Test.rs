#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use wamr_rust_sdk::value::WasmValue;

use File_system::{Create_device, Create_file_system, Memory_device_type};
use Virtual_machine::{
    Environment_type, Function_descriptor_type, Function_descriptors, Instance_type, Module_type,
    Registrable_trait, Runtime_type,
};

pub struct Registrable;

impl Registrable_trait for Registrable {
    fn Get_functions(&self) -> &[Function_descriptor_type] {
        &Functions
    }

    fn Get_name(&self) -> &'static str {
        "Virtual_machine_WASM_test"
    }
}

const Functions: [Function_descriptor_type; 0] = Function_descriptors! {};

#[ignore]
#[test]
fn Integration_test() {
    let Task_instance = Task::Initialize().expect("Failed to initialize task manager");

    Users::Initialize().expect("Failed to initialize users manager");

    Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()))
        .expect("Failed to initialize time manager");

    let Device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::Format(Device.clone(), 512).unwrap();
    let File_system = Create_file_system!(LittleFS::File_system_type::New(Device, 256).unwrap());

    Virtual_file_system::Initialize(File_system).unwrap();

    // Set environment variables
    let Task = Task_instance.Get_current_task_identifier().unwrap();

    Virtual_file_system::Get_instance()
        .Create_directory(&"/Devices", Task)
        .unwrap();

    Drivers::Native::Console::Mount_devices(
        Task_instance.Get_current_task_identifier().unwrap(),
        Virtual_file_system::Get_instance(),
    )
    .unwrap();

    Task_instance
        .Set_environment_variable(Task, "Path", "/:/bin:/usr/bin")
        .unwrap();

    // Load the WASM binary
    let Binary_buffer =
        include_bytes!("./WASM_test/target/wasm32-wasip1/release/Virtual_machine_WASM_test.wasm");

    // Register the functions

    let Runtime = Runtime_type::Builder()
        .Register(&Registrable)
        .Build()
        .unwrap();

    let Standard_in = Virtual_file_system::Get_instance()
        .Open(
            &"/Devices/Standard_in",
            File_system::Mode_type::Read_only.into(),
            Task,
        )
        .expect("Failed to open stdin");
    let Standard_out = Virtual_file_system::Get_instance()
        .Open(
            &"/Devices/Standard_out",
            File_system::Mode_type::Write_only.into(),
            Task,
        )
        .expect("Failed to open stdout");
    let Standard_error = Virtual_file_system::Get_instance()
        .Open(
            &"/Devices/Standard_error",
            File_system::Mode_type::Write_only.into(),
            Task,
        )
        .expect("Failed to open stderr");

    let (Standard_in, Standard_out, Standard_error) = Virtual_file_system::Get_instance()
        .Create_new_task_standard_io(Standard_in, Standard_error, Standard_out, Task, Task, false)
        .unwrap();

    let Module = Module_type::From_buffer(
        &Runtime,
        Binary_buffer.to_vec(),
        "main",
        Standard_in,
        Standard_out,
        Standard_error,
    )
    .unwrap();

    let mut Instance =
        Instance_type::New(&Runtime, &Module, 1024 * 4).expect("Failed to instantiate module");

    let _ =
        Environment_type::From_instance(&Instance).expect("Failed to get execution environment");

    assert_eq!(Instance.Call_main(&vec![]).unwrap(), [WasmValue::Void]);

    assert_eq!(
        Instance
            .Call_export_function("GCD", &vec![WasmValue::I32(9), WasmValue::I32(27)])
            .unwrap(),
        [WasmValue::I32(9)]
    );

    // Test allocation and deallocation

    let Pointer = Instance.Allocate::<u32>(4).unwrap();

    unsafe {
        Pointer.write(1234);

        assert_eq!(1234, Pointer.read());
    }

    Instance.Deallocate(Pointer);
}
