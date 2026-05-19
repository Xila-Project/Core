use xila::abi_declarations::{RawMutex, xila_unlock_mutex};

use crate::EnvironmentContext;

#[no_mangle]
pub extern "C" fn __wasm_mutex_lock(mutex: *mut RawMutex) -> bool {
    unsafe {
        let task = EnvironmentContext::get().get_task();

        xila_unlock_mutex(mutex, task.into())
    }
}

#[no_mangle]
pub extern "C" fn __wasm_mutex_unlock(mutex: *mut RawMutex) -> bool {
    unsafe {
        let task = EnvironmentContext::get().get_task();

        xila_unlock_mutex(mutex, task.into())
    }
}
