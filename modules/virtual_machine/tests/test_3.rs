extern crate alloc;

extern crate abi_definitions;

use abi_context::FileIdentifier;
use alloc::vec;
use executable::build_crate;
use log::information;
use std::fs;
use task::test;
use virtual_machine::{
    Function_descriptors, FunctionDescriptor, Instance, Module, Registrable, Runtime,
};
use wamr_rust_sdk::{RuntimeError, function::Function, value::WasmValue};

drivers_std::memory::instantiate_global_allocator!();

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
    let (standard_in, standard_out, standard_error) = testing::initialize(false).await.split();

    let task_instance = task::get_instance();
    let task = task_instance.get_current_task_identifier().await;

    let binary_path = build_crate(&"virtual_machine_wasm_test").unwrap();
    let binary_buffer = fs::read(&binary_path).expect("Failed to read the binary file");

    abi_context::get_instance()
        .call_abi(async || {
            // Register the functions

            let standard_in = abi_context::get_instance()
                .insert_file(
                    task,
                    standard_in.into_synchronous_file(),
                    Some(FileIdentifier::STANDARD_IN),
                )
                .unwrap();

            let standard_out = abi_context::get_instance()
                .insert_file(
                    task,
                    standard_out.into_synchronous_file(),
                    Some(FileIdentifier::STANDARD_OUT),
                )
                .unwrap();

            let standard_error = abi_context::get_instance()
                .insert_file(
                    task,
                    standard_error.into_synchronous_file(),
                    Some(FileIdentifier::STANDARD_ERROR),
                )
                .unwrap();

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

            let function = Function::find_export_func(instance.get_inner_reference(), "_start")
                .expect("Failed to find _start function");

            loop {
                let result = function.call(instance.get_inner_reference(), &vec![]);

                println!("Result: {result:?}");

                match result {
                    Ok(values) => {
                        if values == [WasmValue::Void] {
                            information!("Function returned without qnything successfully.");
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

                        information!("Caught exception: {}", e.message);
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
