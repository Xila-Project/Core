use xila::abi_declarations::{RawMutex, xila_unlock_mutex};

use crate::EnvironmentContext;

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_mutex_lock(mutex: *mut RawMutex) -> bool {
    unsafe {
        let environment = match EnvironmentContext::get() {
            Some(context) => context,
            None => return false,
        };

        xila_unlock_mutex(mutex, environment.get_task().into())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_mutex_unlock(mutex: *mut RawMutex) -> bool {
    unsafe {
        let environment = match EnvironmentContext::get() {
            Some(context) => context,
            None => return false,
        };

        xila_unlock_mutex(mutex, environment.get_task().into())
    }
}
