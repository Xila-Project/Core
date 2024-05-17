#![allow(non_snake_case)]

pub mod Instance;
pub mod Module;
pub mod Runtime;

pub use Instance::*;
pub use Module::*;
pub use Runtime::*;

#[cfg(test)]
mod tests {

    use wamr_rust_sdk::{function::Function, value::WasmValue};

    use crate::Runtime::Runtime_type;

    use super::*;

    extern "C" fn Test_function(a: i32, b: i32) -> i32 {
        println!("test");
        a + b
    }

    #[test]
    fn Test() {
        std::thread::spawn(|| {
            let mut Runtime_builder = Runtime_type::Builder();

            unsafe {
                Runtime_builder = Runtime_builder
                    .Register_function("Test_function", Test_function as *mut std::ffi::c_void);
            }

            let Runtime = Runtime_builder.Build().unwrap();

            let Buffer =
                include_bytes!("../../../../Test/target/wasm32-unknown-unknown/release/Test.wasm");

            let mut Module = Module_type::From_buffer(&Runtime, Buffer, "main").unwrap();

            let Instance = Instance_type::New(&Runtime, &Module, 1024 * 4).unwrap();

            assert_eq!(
                Instance.Call_main(&vec![WasmValue::I32(0)]).unwrap(),
                WasmValue::I32(0)
            );

            assert_eq!(
                Instance
                    .Call_export_function("gcd", &vec![WasmValue::I32(9), WasmValue::I32(27)])
                    .unwrap(),
                WasmValue::I32(9)
            );
        })
        .join()
        .unwrap();
    }
}
