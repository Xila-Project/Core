#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use Bindings::Task_bindings;
use Virtual_machine::{Data_type, Instantiate_test_environment, WasmValue};

#[test]
fn Integration_test() {
    let Binary_buffer = include_bytes!(
        "../../../target/wasm32-unknown-unknown/release/File_system_bindings_WASM_test.wasm"
    );

    let Task_manager = Task::Manager_type::New();

    let (_Runtime, _Module, Instance) = Instantiate_test_environment(
        Binary_buffer,
        Task_bindings::New(Task_manager.clone()),
        &Data_type::New(),
    );

    assert_eq!(
        Instance.Call_export_function("Test_task", &vec![]).unwrap(),
        WasmValue::I32(42)
    )
}
