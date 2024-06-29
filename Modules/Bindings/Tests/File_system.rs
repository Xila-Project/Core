#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use Bindings::File_system_bindings;
use File_system::{
    Drivers::Native::File_system_type,
    Prelude::{Path_type, Virtual_file_system_type},
};
use Virtual_machine::{Data_type, Instantiate_test_environment, WasmValue};

#[test]
fn Integration_test() {
    let Binary_buffer = include_bytes!(
        "../../../target/wasm32-unknown-unknown/release/File_system_bindings_WASM_test.wasm"
    );

    let Virtual_file_system =
        Virtual_file_system_type::New(Task::Manager_type::New(), Users::Manager_type::New());

    let Native_file_system = File_system_type::New().expect("Failed to create file system");

    let _ = Virtual_file_system.Mount(Box::new(Native_file_system), Path_type::Get_root());

    let (_Runtime, _Module, Instance) = Instantiate_test_environment(
        Binary_buffer,
        File_system_bindings::New(Virtual_file_system.clone()),
        &Data_type::New(),
    );

    assert_eq!(
        Instance
            .Call_export_function("Test_file_system", &vec![])
            .unwrap(),
        WasmValue::I32(0)
    )
}
