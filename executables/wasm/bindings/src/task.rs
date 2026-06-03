use core::ffi::c_int;
use core::ptr::null_mut;
use core::{ffi::c_void, time::Duration};

use xila::log;

use crate::EnvironmentContext;

pub type XilaTaskIdentifier = usize;

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_get_identifier() -> XilaTaskIdentifier {
    unsafe {
        let environment = match EnvironmentContext::get() {
            Some(context) => context,
            None => return 0,
        };

        environment.get_task().into()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_sleep(duration: u64) {
    log::information!("Sleeping for {} ms", duration);
    unsafe {
        match EnvironmentContext::get() {
            Some(context) => context.sleep(Duration::from_millis(duration)),
            None => {
                log::warning!(
                    "Failed to get environment context for sleeping task. Duration: {} ms",
                    duration
                );
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_sleep_exact(duration: u32) {
    log::information!("Sleeping for {} ms", duration);
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
        let environment = match EnvironmentContext::get() {
            Some(context) => context,
            None => return,
        };

        environment.exit();
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
    log::information!("Yielding execution");
    unsafe {
        let environment = match EnvironmentContext::get() {
            Some(context) => context,
            None => return -1,
        };

        environment.yield_now();
    }
    0
}
