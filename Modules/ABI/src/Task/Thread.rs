use core::{ffi::c_void, time::Duration};
use std::ptr::null_mut;

use Task::{Get_instance as Get_task_manager_instance, Manager_type};

pub type Xila_thread_identifier_type = usize;

#[no_mangle]
pub extern "C" fn Xila_get_current_thread_identifier() -> usize {
    Get_task_manager_instance()
        .Get_current_thread_identifier()
        .into()
}

#[no_mangle]
pub extern "C" fn Xila_thread_sleep(Duration: u64) {
    Manager_type::Sleep(Duration::from_millis(Duration));
}

#[no_mangle]
pub extern "C" fn Xila_thread_sleep_exact(_Duration: u32) {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_thread_join(_Thread: usize) -> u32 {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_thread_detach(_Thread: usize) -> u32 {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_thread_exit() {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_thread_get_stack_boundary() -> *mut u8 {
    null_mut()
}

#[no_mangle]
pub extern "C" fn Xila_thread_create(
    _Function: extern "C" fn(*mut c_void) -> *mut c_void,
    _Argument: *mut u8,
    _Stack_size: usize,
    _Thread_identifier: *mut Xila_thread_identifier_type,
) -> u32 {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_thread_begin_blocking_operation() {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_thread_end_blocking_operation() {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_thread_wake_up(_Thread: Xila_thread_identifier_type) -> u32 {
    todo!()
}
