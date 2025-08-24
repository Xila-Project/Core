use core::ffi::{c_int, c_void};

#[unsafe(no_mangle)]
pub extern "C" fn xila_search_binary(
    key: *const c_void,
    base: *const c_void,
    size: usize,
    element_size: usize,
    compare: extern "C" fn(*const c_void, *const c_void) -> c_int,
) -> *mut c_void {
    if key.is_null() || base.is_null() || size == 0 || element_size == 0 {
        return core::ptr::null_mut(); // Invalid parameters
    }

    let mut left: isize = 0;
    let mut right: isize = (size as isize) - 1;

    while left <= right {
        let mid: isize = left + (right - left) / 2;
        let mid_ptr = unsafe { base.add((mid * element_size as isize) as usize) };

        let cmp_result = compare(key, mid_ptr);

        if cmp_result == 0 {
            return mid_ptr as *mut c_void; // Found - return pointer to element
        } else if cmp_result < 0 {
            right = mid - 1;
        } else {
            left = mid + 1;
        }
    }

    core::ptr::null_mut() // Not found
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_sort_quick(
    base: *mut c_void,
    size: usize,
    element_size: usize,
    compare: extern "C" fn(*const c_void, *const c_void) -> c_int,
) {
    if base.is_null() || size == 0 || element_size == 0 {
        return; // Invalid parameters
    }

    if size <= 1 {
        return; // Already sorted
    }

    unsafe {
        quicksort_recursive(base, 0, (size as isize) - 1, element_size, compare);
    }
}

unsafe fn quicksort_recursive(
    base: *mut c_void,
    low: isize,
    high: isize,
    element_size: usize,
    compare: extern "C" fn(*const c_void, *const c_void) -> c_int,
) {
    if low < high {
        let pivot_index = unsafe { partition(base, low, high, element_size, compare) };
        unsafe {
            quicksort_recursive(base, low, pivot_index - 1, element_size, compare);
            quicksort_recursive(base, pivot_index + 1, high, element_size, compare);
        }
    }
}

unsafe fn partition(
    base: *mut c_void,
    low: isize,
    high: isize,
    element_size: usize,
    compare: extern "C" fn(*const c_void, *const c_void) -> c_int,
) -> isize {
    // Choose the rightmost element as pivot
    let pivot_ptr = unsafe { base.add((high * element_size as isize) as usize) };
    let mut i = low - 1; // Index of smaller element

    for j in low..high {
        let j_ptr = unsafe { base.add((j * element_size as isize) as usize) };

        // If current element is smaller than or equal to pivot
        if compare(j_ptr, pivot_ptr) <= 0 {
            i += 1;
            let i_ptr = unsafe { base.add((i * element_size as isize) as usize) };
            unsafe { swap_elements(i_ptr, j_ptr, element_size) };
        }
    }

    // Place pivot in its correct position
    let i_plus_1_ptr = unsafe { base.add(((i + 1) * element_size as isize) as usize) };
    unsafe { swap_elements(i_plus_1_ptr, pivot_ptr, element_size) };

    i + 1
}

unsafe fn swap_elements(a: *mut c_void, b: *mut c_void, element_size: usize) {
    if a == b {
        return; // No need to swap with itself
    }

    let a_bytes = unsafe { core::slice::from_raw_parts_mut(a as *mut u8, element_size) };
    let b_bytes = unsafe { core::slice::from_raw_parts_mut(b as *mut u8, element_size) };

    for i in 0..element_size {
        let temp = a_bytes[i];
        a_bytes[i] = b_bytes[i];
        b_bytes[i] = temp;
    }
}
