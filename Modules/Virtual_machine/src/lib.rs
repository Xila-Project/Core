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
    use std::borrow::Borrow;

    use super::*;

    extern "C" fn test(a: i32, b: i32) -> i32 {
        println!("test");
        a + b
    }

    pub struct Test_s {
        pub a: i32,
        pub b: i32,
    }

    impl Test_s {
        pub fn new() -> Self {
            Test_s { a: 0, b: 0 }
        }

        extern "C" fn add(&mut self, a: i32, b: i32) -> i32 {
            println!("add : {:#16x}", &self as *const _ as usize);
            self.a = a;
            self.b = b;
            self.a + self.b
        }
    }

    extern "C" fn Test_s_add(a: u64, b: i32, c: i32) -> i32 {
        println!("Test_s_add : {:#16x}", a);
        let t = unsafe { &mut *(a as *mut Test_s) };
        t.add(b, c)
    }

    static mut test_s: *mut Test_s = std::ptr::null_mut();

    fn Get_test() -> &'static mut Test_s {
        unsafe { &mut *test_s }
    }

    #[test]
    fn Test() {
        std::thread::spawn(|| {
            let t = Test_s::new();

            //   unsafe {
            //       test_s = &t as *const _ as usize;
            //   }
            //println!("Test_s : {:#16x}", unsafe { test_s });

            let mut Runtime_builder = Runtime::builder().use_system_allocator();

            unsafe {
                Runtime_builder = Runtime_builder
                    .register_host_function("Get_test", Get_test as *mut std::ffi::c_void)
                    .register_host_function("Test_s_add", Test_s_add as *mut std::ffi::c_void)
                    .register_host_function("test", test as *mut std::ffi::c_void);
            }

            let runtime = Runtime_builder.build().unwrap();

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
