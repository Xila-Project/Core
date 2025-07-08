#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

use alloc::vec;

use wamr_rust_sdk::value::WasmValue;

use Drivers::Std::Memory::Memory_manager_type;
use File_system::{Create_device, Create_file_system, Memory_device_type};
use Memory::Instantiate_global_allocator;
use Task::Test;
use Virtual_file_system::{create_default_hierarchy, Mount_static_devices};
use Virtual_machine::{
    Environment_type, Function_descriptor_type, Function_descriptors, Instance_type, Module_type,
    Registrable_trait, Runtime_type,
};

Instantiate_global_allocator!(Memory_manager_type);

pub struct Registrable;

impl Registrable_trait for Registrable {
    fn get_functions(&self) -> &[Function_descriptor_type] {
        &FUNCTIONS
    }

    fn get_name(&self) -> &'static str {
        "Virtual_machine_WASM_test"
    }
}

const FUNCTIONS: [Function_descriptor_type; 0] = Function_descriptors! {};

#[ignore]
#[Test]
async fn integration_test() {
    let task_instance = Task::Initialize();

    static LOGGER: Drivers::Std::Log::Logger_type = Drivers::Std::Log::Logger_type;

    Log::Initialize(&LOGGER).expect("Failed to initialize logger");

    Users::Initialize();

    Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::new()))
        .expect("Failed to initialize time manager");

    let device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    LittleFS::File_system_type::format(device.clone(), 512).unwrap();
    let file_system = Create_file_system!(LittleFS::File_system_type::new(device, 256).unwrap());

    let virtual_file_system = Virtual_file_system::initialize(file_system, None).unwrap();

    // Set environment variables
    let task = task_instance.get_current_task_identifier().await;

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

    task_instance
        .Set_environment_variable(task, "Path", "/:/bin:/usr/bin")
        .await
        .unwrap();

    // Load the WASM binary
    let binary_buffer =
        include_bytes!("./WASM_test/target/wasm32-wasip1/release/Virtual_machine_WASM_test.wasm");

    let standard_in = virtual_file_system
        .open(
            &"/Devices/Standard_in",
            File_system::Mode_type::READ_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stdin");
    let standard_out = virtual_file_system
        .open(
            &"/Devices/Standard_out",
            File_system::Mode_type::WRITE_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stdout");
    let standard_error = virtual_file_system
        .open(
            &"/Devices/Standard_error",
            File_system::Mode_type::WRITE_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stderr");

    let (standard_in, standard_out, standard_error) = virtual_file_system
        .create_new_task_standard_io(standard_in, standard_error, standard_out, task, task, false)
        .await
        .unwrap();

    ABI::get_instance()
        .call_abi(async || {
            // Register the functions

            let runtime = Runtime_type::builder()
                .register(&Registrable)
                .Build()
                .unwrap();

            let module = Module_type::From_buffer(
                &runtime,
                binary_buffer.to_vec(),
                "main",
                standard_in,
                standard_out,
                standard_error,
            )
            .await
            .unwrap();

            let mut instance = Instance_type::New(&runtime, &module, 1024 * 4)
                .expect("Failed to instantiate module");

            let _ = Environment_type::From_instance(&instance)
                .expect("Failed to get execution environment");

            assert_eq!(instance.Call_main(&vec![]).unwrap(), [WasmValue::Void]);

            assert_eq!(
                instance
                    .Call_export_function("GCD", &vec![WasmValue::I32(9), WasmValue::I32(27)])
                    .unwrap(),
                [WasmValue::I32(9)]
            );

            // Test allocation and deallocation

            let pointer = instance.Allocate::<u32>(4).unwrap();

            unsafe {
                pointer.write(1234);

                assert_eq!(1234, pointer.read());
            }

            instance.Deallocate(pointer);
        })
        .await;
}
