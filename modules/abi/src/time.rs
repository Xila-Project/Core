use time::get_instance;

pub type XilaTime = u64;

pub type XilaTimeClockIdentifier = usize;

/// Retrieve the current time since the system startup in microseconds.
///
/// # Returns
///
/// The current time since the system startup in microseconds.
#[unsafe(no_mangle)]
pub extern "C" fn xila_time_get_time_since_startup_microseconds() -> u64 {
    get_instance()
        .get_current_time_since_startup()
        .unwrap_or_default()
        .as_micros() as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_time_get_cpu() -> u64 {
    todo!()
}

/// Retrieve the current time since the system startup in milliseconds.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_time_get_resolution(
    _clock_identifier: XilaTimeClockIdentifier,
    _resolution: *mut XilaTime,
) -> u32 {
    todo!()
}

/// Retrieve the current time since the system startup in milliseconds.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub extern "C" fn xila_time_get_time(
    _clock_identifier: XilaTimeClockIdentifier,
    _precision: u64,
    _time: *mut XilaTime,
) -> u32 {
    todo!()
}
