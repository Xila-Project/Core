#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use Bindings::File_system_bindings;
use File_system::{
    Drivers::Native::{self, File_system_type},
    Prelude::File_system_traits,
};
use Virtual_machine::{Data_type, Instantiate_test_environment, WasmValue};

#[test]
fn Integration_test() {
    let Binary_buffer = include_bytes!(
        "../../../target/wasm32-unknown-unknown/release/File_system_bindings_WASM_test.wasm"
    );

    let mut Native_file_system = File_system_type::New();

    Native_file_system.Initialize().unwrap();

    let User_data = Data_type::New(&Native_file_system);

    let (_Runtime, _Module, Instance) =
        Instantiate_test_environment(Binary_buffer, File_system_bindings {}, &User_data);

    assert_eq!(Instance.Call_main(&vec![]).unwrap(), WasmValue::I32(0))
}
