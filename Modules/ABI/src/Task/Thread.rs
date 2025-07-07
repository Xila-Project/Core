use core::ptr::null_mut;
use core::{ffi::c_void, time::Duration};

use Futures::block_on;
use Task::Manager_type;

use crate::Context;

pub type Xila_thread_identifier_type = usize;

#[no_mangle]
pub extern "C" fn Xila_get_current_thread_identifier() -> usize {
    Context::Get_instance()
        .Get_current_task_identifier()
        .Into_inner() as usize
}

#[no_mangle]
pub extern "C" fn Xila_thread_sleep(Duration: u64) {
    block_on(Manager_type::Sleep(Duration::from_millis(Duration)));
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
    unreachable!("Thread exit is not supported in this environment");
}

#[no_mangle]
pub extern "C" fn Xila_thread_get_stack_boundary() -> *mut u8 {
    null_mut()
}

#[no_mangle]
pub extern "C" fn Xila_thread_create(
    _function: extern "C" fn(*mut c_void) -> *mut c_void,
    _argument: *mut u8,
    _stack_size: usize,
    _thread_identifier: *mut Xila_thread_identifier_type,
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
