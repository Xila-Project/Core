#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use wamr_rust_sdk::value::WasmValue;

use File_system::Drivers::Native::File_system_type;
use Virtual_machine::{
    Data_type, Environment_pointer_type, Environment_type, Function_descriptor_type,
    Function_descriptors, Instantiate_test_environment, Registrable_trait, WASM_pointer,
    WASM_usize,
};

const Testing_slice: [i32; 10] = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

extern "C" fn Test_mutable_slice(
    Raw_environment: Environment_pointer_type,
    Slice: WASM_pointer,
    Size: WASM_usize,
) {
    let Environment = Environment_type::From_raw_pointer(Raw_environment).unwrap();

    let Slice: &mut [i32] = Environment
        .Convert_to_native_mutable_slice(Slice, Size)
        .unwrap();

    assert_eq!(Slice.len(), Testing_slice.len());
    assert_eq!(Slice, &Testing_slice);

    for Element in Slice.iter_mut() {
        *Element = 42;
    }

    assert_eq!(Slice, &[42; 10]);
}

extern "C" fn Test_slice(
    Raw_environment: Environment_pointer_type,
    Slice: WASM_pointer,
    Length: WASM_usize,
) {
    let Environment = Environment_type::From_raw_pointer(Raw_environment).unwrap();

    let Slice: &[i32] = Environment.Convert_to_native_slice(Slice, Length).unwrap();

    assert_eq!(Slice.len(), Testing_slice.len());
    assert_eq!(Slice, Testing_slice);
}

extern "C" fn Test_mutable_string(
    Raw_environment: Environment_pointer_type,
    String: WASM_pointer,
    Length: WASM_pointer,
    Size: WASM_usize,
) {
    let Environment = Environment_type::From_raw_pointer(Raw_environment).unwrap();

    let mut String = Environment
        .Convert_to_native_mutable_string(String, Length, Size)
        .unwrap();

    assert_eq!(String.Get_length(), 5);

    String += " World from WASM!";

    assert_eq!(String.As_str(), "Hello World from WASM!");
}

extern "C" fn Test_string(
    Raw_environment: Environment_pointer_type,
    String: WASM_pointer,
    Length: WASM_usize,
) {
    let Environment = Environment_type::From_raw_pointer(Raw_environment).unwrap();

    let String = Environment
        .Convert_to_native_string(String, Length)
        .unwrap();

    assert_eq!(String, "Hello World from WASM!");
}

#[test]
fn Integration_test() {
    let Binary_buffer = include_bytes!(
        "../../../target/wasm32-unknown-unknown/release/Virtual_machine_WASM_test.wasm"
    );

    pub struct Registrable {}

    impl Registrable_trait for Registrable {
        fn Get_functions(&self) -> &[Function_descriptor_type] {
            &Functions
        }
    }

    const Functions: [Function_descriptor_type; 4] = Function_descriptors! {
        Test_mutable_slice,
        Test_slice,
        Test_mutable_string,
        Test_string
    };

    let File_system = File_system_type::New();

    let User_data = Data_type::New(&File_system);

    let (_Runtime, _Module, Instance) =
        Instantiate_test_environment(Binary_buffer, Registrable {}, &User_data);

    let mut _Execution_environment =
        Environment_type::From_instance(&Instance).expect("Failed to get execution environment");

    assert_eq!(Instance.Call_main(&vec![]).unwrap(), WasmValue::I32(0));

    assert_eq!(
        Instance
            .Call_export_function("GCD", &vec![WasmValue::I32(9), WasmValue::I32(27)])
            .unwrap(),
        WasmValue::I32(9)
    );
}
