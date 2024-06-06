#![allow(non_snake_case)]

use std::{collections::HashMap, fmt::Pointer, os::raw::c_void};

#[link(wasm_import_module = "host")]
extern "C" {
    pub fn Test_mutable_slice(Slice: *mut i32, Size: usize);
    pub fn Test_slice(Slice: *const i32, Length: usize);
    pub fn Test_mutable_string(String: *mut u8, Length: *mut usize, Size: usize);
    pub fn Test_string(String: *const u8, Length: usize);
}

#[export_name = "GCD"]
pub fn GCD(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn Test_passing_mutable_slice() -> Result<(), ()> {
    let mut Slice = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

    unsafe {
        Test_mutable_slice(Slice.as_mut_ptr(), Slice.len());
    }

    if Slice != [42; 10] {
        return Err(());
    }

    Ok(())
}

fn Test_passing_slice() -> Result<(), ()> {
    let Slice = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

    unsafe {
        Test_slice(Slice.as_ptr(), Slice.len());
    }

    Ok(())
}

fn Test_passing_mutable_string() -> Result<(), ()> {
    let mut String = String::from("Hello");

    String.reserve(40);

    let mut String_vector = String.into_bytes();

    let mut String_length = String_vector.len();

    unsafe {
        Test_mutable_string(
            String_vector.as_mut_ptr(),
            &mut String_length as *mut usize,
            String_vector.capacity(),
        );
        String_vector.set_len(String_length);
    }

    let String = unsafe { String::from_utf8_unchecked(String_vector) };

    if String != "Hello World from WASM!" {
        return Err(());
    }

    Ok(())
}

fn Test_passing_string() -> Result<(), ()> {
    let String = "Hello World from WASM!".as_bytes();

    unsafe {
        Test_string(String.as_ptr(), String.len());
    }

    Ok(())
}

static mut Allocator: Option<HashMap<usize, usize>> = None;

/// This function is a C-compatible wrapper around the Rust `Get_allocations_count` function.
/// It is intended to be called from virtual machines that do not have access to Rust's memory management system.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
#[export_name = "Get_allocations_count"]
pub unsafe fn Get_allocations_count() -> usize {
    match unsafe { Allocator.as_ref() } {
        Some(A) => A.len(),
        None => 0,
    }
}

/// This function is a C-compatible wrapper around the Rust `dealloc` function.
/// It is intended to be called from virtual machines that do not have access to Rust's memory management system.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
#[export_name = "free"]
pub unsafe fn Free(Pointer: *mut u8) {
    let A = match unsafe { Allocator.as_mut() } {
        Some(A) => A,
        None => return,
    };

    if Pointer.is_null() {
        return;
    }

    if let Some(Size) = A.remove(&(Pointer as usize)) {
        unsafe {
            std::alloc::dealloc(
                Pointer,
                std::alloc::Layout::from_size_align(Size, 1).unwrap(),
            );
        }
    }
}

/// This function is a C-compatible wrapper around the Rust `malloc` function.
/// It is intended to be called from virtual machines that do not have access to Rust's memory management system.
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
#[export_name = "malloc"]
pub unsafe fn Malloc(Size: usize) -> *mut u8 {
    // - Initialize the Allocator if it is not initialized.
    unsafe {
        if Allocator.is_none() {
            Allocator = Some(HashMap::new());
        }
    }

    let A = unsafe { Allocator.as_mut().unwrap() };

    let Pointer = std::ptr::null_mut();

    if Size == 0 {
        return Pointer;
    }

    let Pointer =
        unsafe { std::alloc::alloc(std::alloc::Layout::from_size_align(Size, 1).unwrap()) };

    if !Pointer.is_null() {
        A.insert(Pointer as usize, Size);
    }

    Pointer
}

fn main() -> Result<(), ()> {
    Test_passing_mutable_slice()?;

    Test_passing_slice()?;

    Test_passing_mutable_string()?;

    Test_passing_string()?;

    Ok(())
}
