use core::ffi::{CStr, c_char};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_print(message: *const c_char) {
    if message.is_null() {
        log::warning!("xila_print called with null message pointer");
        return;
    }

    let message = unsafe { CStr::from_ptr(message).to_string_lossy() };

    log::information!("xila_print: {}", message);
}
