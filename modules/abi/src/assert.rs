use core::ffi::{CStr, c_char};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_assert(condition: bool, message: *const c_char) {
    let message = CStr::from_ptr(message)
        .to_str()
        .expect("Invalid UTF-8 in assertion message");

    if !condition {
        panic!("Assertion failed: {}", message);
    }
}
