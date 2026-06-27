use core::ffi::{CStr, c_char};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_panic(message: *const c_char) {
    if message.is_null() {
        log::warning!("xila_panic called with null message pointer");
        return;
    }

    let message = unsafe { CStr::from_ptr(message).to_string_lossy() };

    log::error!("xila_panic: {}", message);
    panic!("xila_panic: {}", message);
}
