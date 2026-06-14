use crate::XilaTime;

use super::RawMutex;

pub type XilaConditionVariable = usize;

/// This function is used to create a condition variable.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_condition_variable_new(
    _condition_variable: *mut XilaConditionVariable,
) -> bool {
    unimplemented!("xila_condition_variable_new is not implemented yet");
}

/// This function is used to initialize a condition variable.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_condition_variable_initialize(
    _condition_variable: *mut XilaConditionVariable,
) -> bool {
    unimplemented!("xila_condition_variable_initialize is not implemented yet");
}

/// This function is used to delete a condition variable.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_condition_variable_remove(
    _condition_variable: *mut XilaConditionVariable,
) -> bool {
    unimplemented!("xila_condition_variable_remove is not implemented yet");
}

/// This function is used to wait a condition variable.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_condition_variable_wait(
    _condition_variable: *mut XilaConditionVariable,
    _mutex: *mut RawMutex,
) -> bool {
    unimplemented!("xila_condition_variable_wait is not implemented yet");
}

/// This function is used to try wait a condition variable.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_condition_variable_try_wait(
    _condition_variable: *mut XilaConditionVariable,
    _mutex: *mut RawMutex,
    _time: XilaTime,
) -> bool {
    unimplemented!("xila_condition_variable_try_wait is not implemented yet");
}

/// This function is used to signal a condition variable.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_condition_variable_signal(
    _condition_variable: *mut XilaConditionVariable,
) -> bool {
    unimplemented!("xila_condition_variable_signal is not implemented yet");
}

/// This function is used to broadcast a condition variable.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_condition_variable_broadcast(
    _condition_variable: *mut XilaConditionVariable,
) -> bool {
    unimplemented!("xila_condition_variable_broadcast is not implemented yet");
}
