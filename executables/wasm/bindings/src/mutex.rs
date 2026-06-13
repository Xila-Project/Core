use xila::abi_declarations::{RawMutex, xila_unlock_mutex};

use crate::EnvironmentContext;

/// # Safety
/// This function is unsafe because it dereferences a raw pointer, which can lead to undefined behavior if the pointer is null or invalid. It is the caller's responsibility to ensure that the pointer is valid and points to a properly initialized `RawMutex` before calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_mutex_lock(mutex: *mut RawMutex) -> bool {
    unsafe {
        let environment = match EnvironmentContext::get() {
            Some(context) => context,
            None => return false,
        };

        xila_unlock_mutex(mutex, environment.get_task().into())
    }
}

/// # Safety
/// This function is unsafe because it dereferences a raw pointer, which can lead to undefined behavior if the pointer is null or invalid. It is the caller's responsibility to ensure that the pointer is valid and points to a properly initialized `RawMutex` before calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_mutex_unlock(mutex: *mut RawMutex) -> bool {
    unsafe {
        let environment = match EnvironmentContext::get() {
            Some(context) => context,
            None => return false,
        };

        xila_unlock_mutex(mutex, environment.get_task().into())
    }
}
