use core::ffi::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn xila_abort(reason: *const c_char) {
    if reason.is_null() {
        panic!("xila_abort called with no reason");
    } else {
        let reason_str = unsafe { core::ffi::CStr::from_ptr(reason) };
        panic!("xila_abort called: {}", reason_str.to_string_lossy());
    }
}
