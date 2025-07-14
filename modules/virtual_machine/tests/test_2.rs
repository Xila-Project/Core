extern crate alloc;

use alloc::vec;

use wamr_rust_sdk::{function::Function, value::WasmValue, RuntimeError};

use drivers::standard_library::memory::MemoryManager;
use file_system::{create_device, create_file_system, MemoryDevice};
use log::Information;
use memory::instantiate_global_allocator;
use task::test;
use virtual_file_system::{create_default_hierarchy, Mount_static_devices};
use virtual_machine::{
    Environment, FunctionDescriptor, Function_descriptors, Instance, Module, Registrable, Runtime,
};

instantiate_global_allocator!(MemoryManager);

pub struct WasmTest;

impl Registrable for WasmTest {
    fn get_functions(&self) -> &[FunctionDescriptor] {
        &FUNCTIONS
    }

    fn get_name(&self) -> &'static str {
        "Virtual_machine_WASM_test"
    }
}

const FUNCTIONS: [FunctionDescriptor; 0] = Function_descriptors! {};

#[ignore]
#[test]
async fn integration_test() {
    let task_instance = task::initialize();

    static LOGGER: drivers::standard_library::log::Logger = drivers::standard_library::log::Logger;

    log::initialize(&LOGGER).expect("Failed to initialize logger");

    users::initialize();

    time::initialize(create_device!(drivers::native::TimeDriver::new()))
        .expect("Failed to initialize time manager");

    let device = create_device!(MemoryDevice::<512>::new(1024 * 512));

    little_fs::FileSystem::format(device.clone(), 512).unwrap();
    let file_system = create_file_system!(little_fs::FileSystem::new(device, 256).unwrap());

    let virtual_file_system = virtual_file_system::initialize(file_system, None).unwrap();

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
                &"/devices/Standard_in",
                drivers::standard_library::console::StandardInDevice
            ),
            (
                &"/devices/Standard_out",
                drivers::standard_library::console::StandardOutDevice
            ),
            (
                &"/devices/Standard_error",
                drivers::standard_library::console::StandardErrorDevice
            ),
            (&"/devices/Time", drivers::native::TimeDriver),
            (&"/devices/Random", drivers::native::RandomDevice),
            (&"/devices/Null", drivers::core::NullDevice)
        ]
    )
    .await
    .unwrap();

    task_instance
        .set_environment_variable(task, "Path", "/:/bin:/usr/bin")
        .await
        .unwrap();

    // Load the WASM binary
    let binary_buffer =
        include_bytes!("./wasm_test/target/wasm32-wasip1/release/virtual_machine_wasm_test.wasm");

    let standard_in = virtual_file_system
        .open(
            &"/devices/Standard_in",
            file_system::Mode::READ_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stdin");
    let standard_out = virtual_file_system
        .open(
            &"/devices/Standard_out",
            file_system::Mode::WRITE_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stdout");
    let standard_error = virtual_file_system
        .open(
            &"/devices/Standard_error",
            file_system::Mode::WRITE_ONLY.into(),
            task,
        )
        .await
        .expect("Failed to open stderr");

    let (standard_in, standard_out, standard_error) = virtual_file_system
        .create_new_task_standard_io(standard_in, standard_error, standard_out, task, task, false)
        .await
        .unwrap();

    abi::get_instance()
        .call_abi(async || {
            // Register the functions

            let runtime = Runtime::builder().register(&WasmTest).build().unwrap();

            let module = Module::from_buffer(
                &runtime,
                binary_buffer.to_vec(),
                "main",
                standard_in,
                standard_out,
                standard_error,
            )
            .await
            .unwrap();

            let mut instance =
                Instance::new(&runtime, &module, 1024 * 4).expect("Failed to instantiate module");

            let environment =
                Environment::from_instance(&instance).expect("Failed to get execution environment");

            let function = Function::find_export_func(instance.get_inner_reference(), "_start")
                .expect("Failed to find _start function");

            loop {
                // Reset instruction limit before each call
                environment.set_instruction_count_limit(Some(100));

                let result = function.call(instance.get_inner_reference(), &vec![]);

                println!("Result: {result:?}");

                match result {
                    Ok(values) => {
                        if values == [WasmValue::Void] {
                            Information!("Function returned without qnything successfully.");
                        } else {
                            assert_eq!(values.len(), 1);
                            assert_eq!(values[0], WasmValue::Void);
                            break;
                        }
                    }
                    Err(RuntimeError::ExecutionError(e)) => {
                        if e.message != "Exception: instruction limit exceeded" {
                            panic!("Unexpected exception: {}", e.message);
                        }

                        Information!("Caught exception: {}", e.message);
                    }
                    Err(error) => {
                        panic!("Unexpected error: {error:?}");
                    }
                }
            }

            assert_eq!(
                instance
                    .call_export_function("GCD", &vec![WasmValue::I32(9), WasmValue::I32(27)])
                    .unwrap(),
                [WasmValue::I32(9)]
            );

            // Test allocation and deallocation

            let pointer = instance.allocate::<u32>(4).unwrap();

            unsafe {
                pointer.write(1234);

                assert_eq!(1234, pointer.read());
            }

            instance.deallocate(pointer);
        })
        .await;
}
