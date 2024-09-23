use Time::Get_instance;

/// Retrieve the current time since the system startup in seconds.
///
/// # Safety
///
/// This function is unsafe because it might dereference a null pointer.
///
/// # Parameters
///
/// - `Results`: Pointer to a `u128` where the result will be stored.
#[no_mangle]
pub unsafe extern "C" fn Xila_instant_since_startup_microseconds(Results: *mut u128) {
    unsafe {
        *Results = Get_instance().Get_current_time_since_startup().as_micros();
    }
}
