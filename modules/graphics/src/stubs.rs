use core::ffi::{c_char, c_int, c_void};

use abi::{
    xila_memory_allocate_core, xila_memory_copy, xila_memory_deallocate, xila_memory_move,
    xila_memory_reallocate, xila_memory_set, xila_string_compare, xila_string_copy,
    xila_string_copy_bounded, xila_string_duplicate, xila_string_duplicate_bounded,
    xila_string_get_length,
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_memset(destination: *mut c_void, value: u8, size: usize) {
    unsafe { xila_memory_set(destination, value as c_int, size) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_mem_deinit() {
    log::information!("Deinitializing memory manager");
    // This function is a no-op in this stub.
    // In a real implementation, you would clean up memory resources here.
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_mem_init() {
    log::information!("Initializing memory manager");
    // This function is a no-op in this stub.
    // In a real implementation, you would set up memory resources here.
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_malloc_core(size: usize) -> *mut c_void {
    unsafe { xila_memory_allocate_core(size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_free_core(pointer: *mut c_void) {
    xila_memory_deallocate(pointer)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_realloc_core(pointer: *mut c_void, new_size: usize) -> *mut c_void {
    unsafe { xila_memory_reallocate(pointer, new_size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_memcpy(
    destination: *mut c_void,
    source: *const c_void,
    size: usize,
) -> *mut c_void {
    unsafe { xila_memory_copy(destination, source, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_strcmp(str1: *const c_char, str2: *const c_char) -> c_int {
    unsafe { xila_string_compare(str1, str2) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_strlen(string: *const c_char) -> usize {
    unsafe { xila_string_get_length(string) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_snprintf(
    _: *mut c_char,
    _: usize,
    _: *const c_char,
    _: *mut c_void,
) -> c_int {
    log::information!("Formatting string (not implemented in this stub)");
    // This function is not implemented in this stub.
    // In a real implementation, you would handle formatted output here.
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_vsnprintf(
    _: *mut c_char,
    _: usize,
    _: *const c_char,
    _: *mut c_void,
) -> c_int {
    log::information!("Formatting string with variable arguments (not implemented in this stub)");
    // This function is not implemented in this stub.
    // In a real implementation, you would handle formatted output here.
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_strcpy(destination: *mut c_char, source: *const c_char) -> *mut c_char {
    unsafe { xila_string_copy(destination, source) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_strncpy(
    destination: *mut c_char,
    source: *const c_char,
    size: usize,
) -> *mut c_char {
    unsafe { xila_string_copy_bounded(destination, source, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_strdup(string: *const c_char) -> *mut c_char {
    unsafe { xila_string_duplicate(string) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_memmove(
    destination: *mut c_void,
    source: *const c_void,
    size: usize,
) -> *mut c_void {
    unsafe { xila_memory_move(destination, source, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lv_strndup(string: *const c_char, size: usize) -> *mut c_char {
    unsafe { xila_string_duplicate_bounded(string, size) }
}
