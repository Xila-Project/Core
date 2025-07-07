//#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use alloc::vec;

use wamr_rust_sdk::{function::Function, value::WasmValue, RuntimeError};

use Drivers::Std::Memory::Memory_manager_type;
use File_system::{Create_device, Create_file_system, Memory_device_type};
use Log::Information;
use Memory::Instantiate_global_allocator;
use Task::Test;
use Virtual_file_system::{Create_default_hierarchy, Mount_static_devices};
use Virtual_machine::{
    Environment_type, Function_descriptor_type, Function_descriptors, Instance_type, Module_type,
    Registrable_trait, Runtime_type,
};

Instantiate_global_allocator!(Memory_manager_type);

pub struct Registrable;

impl Registrable_trait for Registrable {
    fn get_functions(&self) -> &[Function_descriptor_type] {
        &Functions
    }

    fn Get_name(&self) -> &'static str {
        "Virtual_machine_WASM_test"
    }
}

const Functions: [Function_descriptor_type; 0] = Function_descriptors! {};

#[ignore]
#[Test]
async fn Integration_test() {
    let Task_instance = Task::Initialize();

    static Logger: Drivers::Std::Log::Logger_type = Drivers::Std::Log::Logger_type;

    Log::Initialize(&Logger).expect("Failed to initialize logger");

    Users::Initialize();

    Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::new()))
        .expect("Failed to initialize time manager");

    let Device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::Format(Device.clone(), 512).unwrap();
    let File_system = Create_file_system!(LittleFS::File_system_type::new(Device, 256).unwrap());

    let Virtual_file_system = Virtual_file_system::Initialize(File_system, None).unwrap();

    // Set environment variables
    let Task = Task_instance.Get_current_task_identifier().await;

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

    Task_instance
        .Set_environment_variable(Task, "Path", "/:/bin:/usr/bin")
        .await
        .unwrap();

    // Load the WASM binary
    let Binary_buffer =
        include_bytes!("./WASM_test/target/wasm32-wasip1/release/Virtual_machine_WASM_test.wasm");

    let Standard_in = Virtual_file_system
        .Open(
            &"/Devices/Standard_in",
            File_system::Mode_type::READ_ONLY.into(),
            Task,
        )
        .await
        .expect("Failed to open stdin");
    let Standard_out = Virtual_file_system
        .Open(
            &"/Devices/Standard_out",
            File_system::Mode_type::WRITE_ONLY.into(),
            Task,
        )
        .await
        .expect("Failed to open stdout");
    let Standard_error = Virtual_file_system
        .Open(
            &"/Devices/Standard_error",
            File_system::Mode_type::WRITE_ONLY.into(),
            Task,
        )
        .await
        .expect("Failed to open stderr");

    let (Standard_in, Standard_out, Standard_error) = Virtual_file_system
        .Create_new_task_standard_io(Standard_in, Standard_error, Standard_out, Task, Task, false)
        .await
        .unwrap();

    ABI::Get_instance()
        .Call_ABI(async || {
            // Register the functions

            let Runtime = Runtime_type::builder()
                .Register(&Registrable)
                .Build()
                .unwrap();

            let Module = Module_type::From_buffer(
                &Runtime,
                Binary_buffer.to_vec(),
                "main",
                Standard_in,
                Standard_out,
                Standard_error,
            )
            .await
            .unwrap();

            let mut Instance = Instance_type::New(&Runtime, &Module, 1024 * 4)
                .expect("Failed to instantiate module");

            let Environment = Environment_type::From_instance(&Instance)
                .expect("Failed to get execution environment");

            let Function = Function::find_export_func(Instance.Get_inner_reference(), "_start")
                .expect("Failed to find _start function");

            loop {
                // Reset instruction limit before each call
                Environment.Set_instruction_count_limit(Some(100));

                let Result = Function.call(Instance.Get_inner_reference(), &vec![]);

                println!("Result: {Result:?}");

                match Result {
                    Ok(Values) => {
                        if Values == [WasmValue::Void] {
                            Information!("Function returned without qnything successfully.");
                        } else {
                            assert_eq!(Values.len(), 1);
                            assert_eq!(Values[0], WasmValue::Void);
                            break;
                        }
                    }
                    Err(RuntimeError::ExecutionError(E)) => {
                        if E.message != "Exception: instruction limit exceeded" {
                            panic!("Unexpected exception: {}", E.message);
                        }

                        Information!("Caught exception: {}", E.message);
                    }
                    Err(Error) => {
                        panic!("Unexpected error: {Error:?}");
                    }
                }
            }

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
        })
        .await;
}
