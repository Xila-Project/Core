use Time::Get_instance;

pub type Xila_time_type = u64;

/// Retrieve the current time since the system startup in microseconds.
///
/// # Returns
///
/// The current time since the system startup in microseconds.
#[no_mangle]
pub extern "C" fn Xila_time_get_time_since_startup_microseconds() -> u64 {
    Get_instance()
        .Get_current_time_since_startup()
        .unwrap_or_default()
        .As_microseconds() as u64
}
