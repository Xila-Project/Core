#![allow(non_upper_case_globals)]

use std::{num::NonZeroU32, sync::RwLock};

use Binding_tool::Bind_function_WASM;

#[Bind_function_WASM]
fn New_task(Name: &str, Stack_size: u32, Function: u32) -> Result<(), NonZeroU32> {}

#[Bind_function_WASM]
fn Sleep(Duration: u64) {}

fn Test_function() {
    *Test_variable.write().unwrap() = 42;
}

static Test_variable: RwLock<u32> = RwLock::new(0);

#[no_mangle]
fn Test_task() -> u32 {
    let _ = New_task("Test", 4096, Test_function as usize as u32);

    Sleep(1);

    *Test_variable.read().unwrap()
}
