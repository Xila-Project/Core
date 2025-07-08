use Time::get_instance;

pub type Xila_time_type = u64;

pub type Xila_time_clock_identifier_type = usize;

/// Retrieve the current time since the system startup in microseconds.
///
/// # Returns
///
/// The current time since the system startup in microseconds.
#[no_mangle]
pub extern "C" fn Xila_time_get_time_since_startup_microseconds() -> u64 {
    get_instance()
        .get_current_time_since_startup()
        .unwrap_or_default()
        .As_microseconds() as u64
}

#[no_mangle]
pub extern "C" fn Xila_time_get_cpu() -> u64 {
    todo!()
}

/// Retrieve the current time since the system startup in milliseconds.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_time_get_resolution(
    _clock_identifier: Xila_time_clock_identifier_type,
    _resolution: *mut Xila_time_type,
) -> u32 {
    todo!()
}

/// Retrieve the current time since the system startup in milliseconds.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub extern "C" fn Xila_time_get_time(
    _clock_identifier: Xila_time_clock_identifier_type,
    _precision: u64,
    _time: *mut Xila_time_type,
) -> u32 {
    todo!()
}
