use std::{ptr::null_mut, time::Duration};

use Task::{Get_instance as Get_task_manager_instance, Manager_type};

#[no_mangle]
pub extern "C" fn Xila_get_thread_stack_boundary() -> *const u8 {
    null_mut()
}

#[no_mangle]
pub extern "C" fn Xila_get_current_thread_identifier() -> usize {
    Get_task_manager_instance()
        .Get_current_thread_identifier()
        .into()
}

#[no_mangle]
pub extern "C" fn Xila_sleep(Duration: u64) {
    Manager_type::Sleep(Duration::from_secs(Duration));
}
