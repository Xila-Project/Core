use core::ffi::{c_double, c_int};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_is_nan(value: c_double) -> bool {
    c_double::is_nan(value)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_get_absolute_value(value: c_int) -> c_int {
    c_int::abs(value)
}
