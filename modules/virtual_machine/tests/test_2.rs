extern crate abi_definitions;
extern crate alloc;
extern crate std;

use executable::build_crate;
use std::fs;
use task::test;
use virtual_machine::{Function_descriptors, FunctionDescriptor, Registrable};

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
async fn integration_test_2() {
    let standard = testing::initialize(false, false).await.split();

    let task_instance = task::get_instance();
    let task = task_instance.get_current_task_identifier().await;

    let binary_path = build_crate(&"virtual_machine_wasm_test").unwrap();
    let binary_buffer = fs::read(&binary_path).expect("Failed to read the binary file");

    let virtual_machine = virtual_machine::initialize(&[&WasmTest]);

    virtual_machine
        .execute(
            binary_buffer.to_vec(),
            4 * 1024,
            standard,
            None,
            vec![],
            task,
        )
        .await
        .unwrap();
}
