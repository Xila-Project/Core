#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod Error;
pub mod Execution_environment;
pub mod Instance;
pub mod Module;
pub mod Runtime;

pub use Error::*;
pub use Execution_environment::*;
pub use Instance::*;
pub use Module::*;
pub use Runtime::*;

type WASM_pointer = u32;
type WASM_usize = u32;

#[cfg(test)]
mod tests {

    use wamr_rust_sdk::value::WasmValue;
    use Shared::Mutable_slice_type;

    use crate::Runtime::Runtime_type;

    use super::*;

    const Testing_slice: [i32; 10] = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

    extern "C" fn Test_mutable_slice(
        Raw_environment: Environment_pointer_type,
        Slice: WASM_pointer,
        Size: WASM_usize,
    ) {
        let Environment = Environment_type::From_raw_pointer(Raw_environment).unwrap();

        let mut Slice: &mut [i32] = Environment
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
    fn Test() {
        std::thread::spawn(|| {
            let mut Runtime_builder = Runtime_type::Builder();

            Runtime_builder = Runtime_builder
                .Register_function(
                    "Test_mutable_slice",
                    Test_mutable_slice as *mut std::ffi::c_void,
                )
                .Register_function("Test_slice", Test_slice as *mut std::ffi::c_void)
                .Register_function(
                    "Test_mutable_string",
                    Test_mutable_string as *mut std::ffi::c_void,
                )
                .Register_function("Test_string", Test_string as *mut std::ffi::c_void);

            let Runtime = Runtime_builder.Build().unwrap();

            let Buffer =
                include_bytes!("../../../target/wasm32-unknown-unknown/release/WASM_test.wasm");

            let Module = Module_type::From_buffer(&Runtime, Buffer, "main").unwrap();

            let Instance = Instance_type::New(&Runtime, &Module, 1024 * 4).unwrap();

            let mut Execution_environment = Environment_type::From_instance(&Instance)
                .expect("Failed to get execution environment");

            assert_eq!(
                Instance.Call_main(&vec![WasmValue::I32(0)]).unwrap(),
                WasmValue::I32(0)
            );

            assert_eq!(
                Instance
                    .Call_export_function("GCD", &vec![WasmValue::I32(9), WasmValue::I32(27)])
                    .unwrap(),
                WasmValue::I32(9)
            );

            let mut User_data: u64 = 0123456789;

            unsafe {
                Execution_environment.Set_user_data(&mut User_data);

                assert_eq!(
                    *Execution_environment.Get_user_data::<i64>().unwrap(),
                    0123456789
                );
            }

            let mut User_data: u64 = 9876543210;

            unsafe {
                Execution_environment.Set_user_data(&mut User_data);

                assert_eq!(
                    *Execution_environment.Get_user_data::<i64>().unwrap(),
                    9876543210
                );
            }
        })
        .join()
        .unwrap();
    }
}
