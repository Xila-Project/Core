use core::ffi::c_int;
use core::ptr::null_mut;
use core::{ffi::c_void, time::Duration};

pub type XilaTaskIdentifier = usize;

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_get_identifier() -> XilaTaskIdentifier {
    global_context::get_current_environment_context_synchronous()
        .expect("Failed to get current environment context")
        .get_current_task_identifier()
        .into_inner() as XilaTaskIdentifier
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_sleep(duration: u64) {
    global_context::get_current_environment_context_synchronous()
        .expect("Failed to get current environment context")
        .sleep(Duration::from_millis(duration));
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
    unreachable!("Thread exit is not supported in this environment");
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
    _thread_identifier: *mut XilaThreadIdentifier,
) -> u32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_begin_blocking_operation() {}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_end_blocking_operation() {}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_wake_up(_thread: XilaThreadIdentifier) -> u32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_task_yield() -> c_int {
    global_context::get_current_environment_context_synchronous()
        .expect("Failed to get current environment context")
        .yield_now();

    0
}
