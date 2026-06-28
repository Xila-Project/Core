use core::ffi::{c_char, c_int, c_void};
use core::{ptr, slice};

/// C-compatible binary search implementation (equivalent to `bsearch`).
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. The caller must ensure
/// that `base` points to a valid array of `nmemb` elements, each of `size` bytes,
/// and that `compar` is a valid, non-null function pointer.
///
/// # Credits
///
/// Rust tranlslation of [picolibc `bsearch`](https://github.com/picolibc/picolibc/blob/main/libc/search/bsearch.c).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_binary_search(
    key: *const c_void,
    base: *const c_void,
    nmemb: usize,
    size: usize,
    compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
) -> *mut c_void {
    // Handle the edge cases or if the comparison function is a null pointer
    if nmemb == 0 || size == 0 || compar.is_none() {
        return ptr::null_mut();
    }

    // Unwrap the function pointer securely
    let compar_fn = unsafe { compar.unwrap_unchecked() };

    let mut lower = 0;
    let mut upper = nmemb;

    while lower < upper {
        // Prevent potential overflow by using the midpoint formula
        let index = lower + (upper - lower) / 2;

        // Calculate the address of the current element using byte offset math
        let current = unsafe { (base as *const c_char).add(index * size) } as *const c_void;

        // Perform the comparison
        let result = unsafe { compar_fn(key, current) };

        if result < 0 {
            upper = index;
        } else if result > 0 {
            lower = index + 1;
        } else {
            // Found the element; cast the read-only pointer to a mutable void pointer
            return current as *mut c_void;
        }
    }

    ptr::null_mut()
}

/// C-compatible qsort using an allocation-free Heapsort algorithm.
///
/// # Safety
/// The caller must guarantee that `array` points to valid memory containing
/// `element_count` elements of `element_size` bytes each.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_quick_sort(
    array: *mut c_void,
    element_count: usize,
    element_size: usize,
    compare_function: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
) {
    if array.is_null() || element_count <= 1 || element_size == 0 {
        return;
    }

    let comp = match compare_function {
        Some(f) => f,
        None => return,
    };

    let base_ptr = array as *mut u8;

    // Helper closure to swap two elements entirely in-place byte-by-byte
    // without needing a dynamic scratch space buffer.
    let swap = |i: usize, j: usize| {
        if i == j {
            return;
        }

        unsafe {
            let mut p1 = base_ptr.add(i * element_size);
            let mut p2 = base_ptr.add(j * element_size);
            for _ in 0..element_size {
                let tmp = *p1;
                *p1 = *p2;
                *p2 = tmp;
                p1 = p1.add(1);
                p2 = p2.add(1);
            }
        }
    };

    // Helper closure to compare two elements by index
    let is_less = |i: usize, j: usize| -> bool {
        unsafe {
            let ptr_i = base_ptr.add(i * element_size) as *const c_void;
            let ptr_j = base_ptr.add(j * element_size) as *const c_void;
            comp(ptr_i, ptr_j) < 0
        }
    };

    // Sink/Sift-down function to maintain the Heap property
    let sift_down = |start: usize, end: usize| {
        let mut root = start;
        while root * 2 + 1 < end {
            let child = root * 2 + 1;
            let mut swap_target = root;

            if is_less(swap_target, child) {
                swap_target = child;
            }
            if child + 1 < end && is_less(swap_target, child + 1) {
                swap_target = child + 1;
            }
            if swap_target == root {
                return;
            } else {
                swap(root, swap_target);
                root = swap_target;
            }
        }
    };

    // Step 1: Build the heap (rearrange array)
    let mut i = element_count / 2;
    while i > 0 {
        i -= 1;
        sift_down(i, element_count);
    }

    // Step 2: Pop elements from the heap one by one into the sorted tail
    let mut end = element_count;
    while end > 1 {
        end -= 1;
        swap(0, end); // Move current max to the end
        sift_down(0, end); // Maintain heap property for remaining elements
    }
}
