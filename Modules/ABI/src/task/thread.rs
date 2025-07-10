use core::ptr::null_mut;
use core::{ffi::c_void, time::Duration};

use futures::block_on;
use task::Manager;

use crate::context;

pub type XilaThreadIdentifier = usize;

#[no_mangle]
pub extern "C" fn xila_get_current_thread_identifier() -> usize {
    context::get_instance()
        .get_current_task_identifier()
        .into_inner() as usize
}

#[no_mangle]
pub extern "C" fn xila_thread_sleep(duration: u64) {
    block_on(Manager::sleep(Duration::from_millis(duration)));
}

#[no_mangle]
pub extern "C" fn xila_thread_sleep_exact(_duration: u32) {
    todo!()
}

#[no_mangle]
pub extern "C" fn xila_thread_join(_thread: usize) -> u32 {
    todo!()
}

#[no_mangle]
pub extern "C" fn xila_thread_detach(_thread: usize) -> u32 {
    todo!()
}

#[no_mangle]
pub extern "C" fn xila_thread_exit() {
    unreachable!("Thread exit is not supported in this environment");
}

#[no_mangle]
pub extern "C" fn xila_thread_get_stack_boundary() -> *mut u8 {
    null_mut()
}

#[no_mangle]
pub extern "C" fn xila_thread_create(
    _function: extern "C" fn(*mut c_void) -> *mut c_void,
    _argument: *mut u8,
    _stack_size: usize,
    _thread_identifier: *mut XilaThreadIdentifier,
) -> u32 {
    todo!()
}

#[no_mangle]
pub extern "C" fn xila_thread_begin_blocking_operation() {
    todo!()
}

#[no_mangle]
pub extern "C" fn xila_thread_end_blocking_operation() {
    todo!()
}

#[no_mangle]
pub extern "C" fn xila_thread_wake_up(_thread: XilaThreadIdentifier) -> u32 {
    todo!()
}
