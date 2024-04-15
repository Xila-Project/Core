#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::path::PathBuf;
use std::time::Duration;
use wamr_rust_sdk::{
    function::Function, host_function, instance::Instance, module::Module, runtime::Runtime,
    value::WasmValue, wasi_context::WasiCtxBuilder, RuntimeError,
};

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn test(a: i32, b: i32) -> i32 {
        println!("test");
        a + b
    }

    #[test]
    fn Test() {
        std::thread::spawn(|| {
            let runtime = Runtime::builder()
                .use_system_allocator()
                .register_host_function("test", test as *mut std::ffi::c_void)
                .build()
                .unwrap();

            //let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            //d.push("../Test/target/wasm32-wasi/debug/Test.wasm");
            //let mut module = Module::from_file(&runtime, d.as_path()).unwrap();

            let buffer =
                include_bytes!("../../../../Test/target/wasm32-unknown-unknown/release/Test.wasm");

            let mut module = Module::from_buf(&runtime, buffer, "main").unwrap();

            //let wasi_ctx = WasiCtxBuilder::new()
            //    .set_pre_open_path(vec!["."], vec![])
            //    .build();

            //module.set_wasi_context(wasi_ctx);

            let instance = Instance::new(&runtime, &module, 1024 * 4).unwrap();

            let function = Function::find_export_func(&instance, "gcd").unwrap();

            let params: Vec<WasmValue> = vec![WasmValue::I32(9), WasmValue::I32(27)];

            let result = function.call(&instance, &params).unwrap();
            assert_eq!(result, WasmValue::I32(9));
        })
        .join()
        .unwrap();
    }
}
