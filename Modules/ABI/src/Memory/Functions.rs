#![allow(dead_code)]

extern crate alloc;

use alloc::alloc::{alloc, dealloc, realloc};
use alloc::collections::BTreeMap;
use core::alloc::Layout;
use core::{ffi::c_void, mem::size_of, ptr::NonNull};

use std::sync::RwLock;

use Memory::{Allocator, Layout_type, Memory_allocator_trait};

use super::{Xila_memory_flags_type, Xila_memory_protection_type};

// - Memory

// - - Allocation

static Allocations_table: RwLock<BTreeMap<usize, usize>> = RwLock::new(BTreeMap::new());

#[no_mangle]
pub extern "C" fn Xila_memory_allocate(Size: usize) -> *mut c_void {
    let Layout =
        Layout::from_size_align(Size, size_of::<usize>()).expect("Failed to create layout");

    let Pointer = unsafe { alloc(Layout) };

    if !Pointer.is_null() {
        Allocations_table
            .write()
            .expect("Failed to write to Allocations table")
            .insert(Pointer as usize, Size);
    }

    Pointer as *mut c_void
}

#[no_mangle]
pub extern "C" fn Xila_memory_deallocate(Pointer: *mut c_void) {
    if Pointer.is_null() {
        return;
    }

    let Size = match Allocations_table
        .write()
        .expect("Failed to write to Allocations table")
        .remove(&(Pointer as usize))
    {
        Some(Size) => Size,
        None => return,
    };

    unsafe {
        let Layout = Layout::from_size_align(Size, size_of::<usize>()).unwrap();
        dealloc(Pointer as *mut u8, Layout);
    }
}

#[no_mangle]
pub extern "C" fn Xila_memory_reallocate(Pointer: *mut c_void, Size: usize) -> *mut c_void {
    if Pointer.is_null() {
        return Xila_memory_allocate(Size);
    }

    let Old_size = Allocations_table
        .write()
        .expect("Failed to read from Allocations table")
        .remove(&(Pointer as usize));

    if let Some(Old_size) = Old_size {
        let Layout = Layout::from_size_align(Old_size, size_of::<usize>()).unwrap();

        let New_pointer = unsafe { realloc(Pointer as *mut u8, Layout, Size) };

        if !New_pointer.is_null() {
            Allocations_table
                .write()
                .expect("Failed to write to Allocations table")
                .insert(New_pointer as usize, Size);
        }

        return New_pointer as *mut c_void;
    }

    Xila_memory_allocate(Size)
}

/// This function is used to allocate a memory region.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the memory allocator fails to allocate the memory region.
#[no_mangle]
pub unsafe extern "C" fn Xila_memory_allocate_custom(
    Hint_address: *mut c_void,
    Size: usize,
    Alignment: u8,
    Protection: Xila_memory_protection_type,
    Flags: Xila_memory_flags_type,
) -> *mut c_void {
    println!(
        "Allocating memory custom : {:?} : {} : {} : {:?} : {:?}",
        Hint_address, Size, Alignment, Protection, Flags
    );

    let Hint_address = if Hint_address.is_null() {
        None
    } else {
        Some(NonNull::new_unchecked(Hint_address as *mut u8))
    };

    let Layout = Layout_type::New(Size, Alignment);

    let Flags = Memory::Flags_type::From_u8(Flags);

    let Pointer = Allocator.Allocate_custom(Hint_address, Layout, Protection.into(), Flags);

    println!("Allocated memory custom : {:?}", Pointer);

    match Pointer {
        Some(Pointer) => Pointer.as_ptr() as *mut c_void,
        None => std::ptr::null_mut(),
    }
}

/// This function is used to deallocate a memory region.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the memory allocator fails to deallocate the memory region.
#[no_mangle]
pub unsafe extern "C" fn Xila_memory_deallocate_custom(
    Pointer: *mut c_void,
    Length: usize,
) -> bool {
    println!("Deallocating memory custom : {:p} : {}", Pointer, Length);

    let Pointer =
        NonNull::new(Pointer as *mut u8).expect("Failed to deallocate memory, pointer is null");

    Allocator.Deallocate_custom(Pointer, Length)
}

/// This function is used to set the protection of a memory region.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the memory allocator fails to set the protection of the memory region.
#[no_mangle]
pub unsafe extern "C" fn Xila_memory_protect(
    Pointer: *mut c_void,
    Length: usize,
    Protection: Xila_memory_protection_type,
) -> bool {
    println!(
        "Protecting memory : {:p} : {} : {:?}",
        Pointer, Length, Protection
    );

    let Pointer =
        NonNull::new(Pointer as *mut u8).expect("Failed to protect memory, pointer is null");

    let Protection = Memory::Protection_type::From_u8(Protection);

    Allocator.Protect(Pointer, Length, Protection)
}

#[no_mangle]
pub extern "C" fn Xila_memory_get_page_size() -> usize {
    println!("Getting page size");

    Allocator.Get_page_size()
}
