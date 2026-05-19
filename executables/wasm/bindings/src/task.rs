use core::ffi::c_int;
use core::ptr::null_mut;
use core::{ffi::c_void, time::Duration};

use crate::host::Context;

pub type XilaTaskIdentifier = usize;

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_get_identifier() -> XilaTaskIdentifier {
    unsafe { Context::get_global().get_task().into() }
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_sleep(duration: u64) {
    unsafe {
        Context::get_global().sleep(Duration::from_millis(duration));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_sleep_exact(duration: u32) {
    __wasm_task_sleep(duration as u64);
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_join(_thread: usize) -> u32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_detach(_thread: usize) -> u32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_exit() {
    unsafe {
        Context::get_global().exit();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_get_stack_boundary() -> *mut u8 {
    null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_create(
    _function: extern "C" fn(*mut c_void) -> *mut c_void,
    _argument: *mut u8,
    _stack_size: usize,
    _thread_identifier: *mut XilaTaskIdentifier,
) -> u32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_begin_blocking_operation() {}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_end_blocking_operation() {}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_wake_up(_thread: XilaTaskIdentifier) -> u32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_yield() -> c_int {
    unsafe {
        Context::get_global().yield_now();
    }
    0
}
