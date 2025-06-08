#![allow(dead_code)]

use alloc::collections::BTreeMap;
use core::ptr::null_mut;
use core::{ffi::c_void, ptr::NonNull};
use Futures::block_on;
use Log::Warning;
use Synchronization::blocking_mutex::raw::CriticalSectionRawMutex;

use Synchronization::rwlock::RwLock;

use Memory::{Capabilities_type, Layout_type};

pub type Xila_memory_protection_type = u8;

#[no_mangle]
pub static Xila_memory_protection_read: u8 = Memory::Protection_type::Read_bit;
#[no_mangle]
pub static Xila_memory_protection_write: u8 = Memory::Protection_type::Write_bit;
#[no_mangle]
pub static Xila_memory_protection_execute: u8 = Memory::Protection_type::Execute_bit;

pub type Xila_memory_capabilities_type = u8;
#[no_mangle]
pub static Xila_memory_capabilities_execute: Xila_memory_capabilities_type =
    Memory::Capabilities_type::Executable_flag;
#[no_mangle]
pub static Xila_memory_capabilities_direct_memory_access: Xila_memory_capabilities_type =
    Memory::Capabilities_type::Direct_memory_access_flag;
#[no_mangle]
pub static Xila_memory_capabilities_none: Xila_memory_capabilities_type = 0;

// - Memory

// - - Allocation

pub fn Into_pointer<F, P>(Function: F) -> *mut c_void
where
    F: FnOnce() -> Option<NonNull<P>>,
{
    match Function() {
        Some(Pointer) => Pointer.as_ptr() as *mut c_void,
        None => null_mut(),
    }
}

static Allocations_table: RwLock<CriticalSectionRawMutex, BTreeMap<usize, Layout_type>> =
    RwLock::new(BTreeMap::new());

// Macro to write to allocations table
macro_rules! Write_allocations_table {
    () => {
        block_on(Allocations_table.write())
    };
}

#[no_mangle]
pub extern "C" fn Xila_memory_deallocate(Pointer: *mut c_void) {
    if Pointer.is_null() {
        return;
    }

    let Layout = match Write_allocations_table!().remove(&(Pointer as usize)) {
        Some(Size) => Size,
        None => return,
    };

    unsafe {
        Memory::Get_instance().Deallocate(
            NonNull::new(Pointer as *mut u8).expect("Failed to deallocate memory, pointer is null"),
            Layout,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn Xila_memory_reallocate(Pointer: *mut c_void, Size: usize) -> *mut c_void {
    Into_pointer(|| {
        let Pointer = NonNull::new(Pointer as *mut u8);

        let mut Allocation_table = block_on(Allocations_table.write());

        let Old_layout = match Pointer {
            None => Layout_type::from_size_align(Size, 1)
                .expect("Failed to create layout for memory reallocation"),
            Some(Pointer) =>
            // Get the layout from the allocation table using the pointer's address
            {
                Allocation_table
                    .get(&(Pointer.as_ptr() as usize))
                    .cloned()?
            }
        };

        let New_layout = Layout_type::from_size_align(Size, Old_layout.align())
            .expect("Failed to create layout for memory reallocation");

        let Allocated = Memory::Get_instance().Reallocate(Pointer, Old_layout, New_layout)?;

        Allocation_table.insert(Allocated.as_ptr() as usize, New_layout);

        Some(Allocated)
    })
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
pub unsafe extern "C" fn Xila_memory_allocate(
    Hint_address: *mut c_void,
    Size: usize,
    Alignment: usize,
    Capabilities: Xila_memory_capabilities_type,
) -> *mut c_void {
    Into_pointer(|| {
        let Hint_address = if Hint_address.is_null() {
            None
        } else {
            Some(NonNull::new_unchecked(Hint_address as *mut u8))
        };

        let Layout = Layout_type::from_size_align(Size, Alignment)
            .expect("Failed to create layout for memory allocation");

        let Capabilities = Capabilities_type::From_u8(Capabilities);

        Memory::Get_instance().Allocate(Capabilities, Layout)
    })
}

#[no_mangle]
pub extern "C" fn Xila_memory_get_page_size() -> usize {
    Memory::Get_instance().Get_page_size()
}

#[no_mangle]
pub extern "C" fn Xila_memory_flush_data_cache() {
    Memory::Get_instance().Flush_data_cache();
}

#[no_mangle]
pub extern "C" fn Xila_memory_flush_instruction_cache(_Address: *mut c_void, _Size: usize) {
    let Address = NonNull::new(_Address as *mut u8).expect("Failed to flush instruction cache");

    Memory::Get_instance().Flush_instruction_cache(Address, _Size);
}

#[cfg(test)]
mod Tests {
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn Test_allocate_deallocate_basic() {
        unsafe {
            // Test basic allocation and deallocation
            let size = 128;
            let alignment = 8;
            let capabilities = 0; // Basic capabilities
            let hint_address = core::ptr::null_mut();

            let pointer = Xila_memory_allocate(hint_address, size, alignment, capabilities);
            assert!(!pointer.is_null(), "Memory allocation should succeed");

            // Write and read to verify the memory is accessible
            let ptr = pointer as *mut u8;
            for i in 0..size {
                *ptr.add(i) = (i % 256) as u8;
            }

            for i in 0..size {
                assert_eq!(
                    *ptr.add(i),
                    (i % 256) as u8,
                    "Memory should be readable and writable"
                );
            }

            // Deallocate the memory
            Xila_memory_deallocate(pointer);
        }
    }

    #[test]
    fn test_allocate_zero_size() {
        unsafe {
            // Test allocation with zero size
            let size = 0;
            let alignment = 8;
            let capabilities = 0;
            let hint_address = core::ptr::null_mut();

            let pointer = Xila_memory_allocate(hint_address, size, alignment, capabilities);
            // Zero-size allocation might return null or a valid pointer, both are acceptable
            if !pointer.is_null() {
                Xila_memory_deallocate(pointer);
            }
        }
    }

    #[test]
    fn test_allocate_large_alignment() {
        unsafe {
            // Test allocation with large alignment
            let size = 256;
            let alignment = 64; // 64-byte alignment
            let capabilities = 0;
            let hint_address = core::ptr::null_mut();

            let pointer = Xila_memory_allocate(hint_address, size, alignment, capabilities);
            assert!(
                !pointer.is_null(),
                "Large alignment allocation should succeed"
            );

            // Verify alignment
            let addr = pointer as usize;
            assert_eq!(addr % alignment, 0, "Pointer should be properly aligned");

            Xila_memory_deallocate(pointer);
        }
    }

    #[test]
    fn test_allocate_executable_capability() {
        unsafe {
            // Test allocation with executable capability
            let size = 128;
            let alignment = 8;
            let capabilities = Xila_memory_capabilities_execute;
            let hint_address = core::ptr::null_mut();

            let pointer = Xila_memory_allocate(hint_address, size, alignment, capabilities);
            assert!(
                !pointer.is_null(),
                "Executable memory allocation should succeed"
            );

            Xila_memory_deallocate(pointer);
        }
    }

    #[test]
    #[ignore = "Requires specific hardware support for DMA"]
    fn test_allocate_dma_capability() {
        unsafe {
            // Test allocation with DMA capability
            let size = 128;
            let alignment = 8;
            let capabilities = Xila_memory_capabilities_direct_memory_access;
            let hint_address = core::ptr::null_mut();

            let pointer = Xila_memory_allocate(hint_address, size, alignment, capabilities);
            assert!(!pointer.is_null(), "DMA memory allocation should succeed");

            Xila_memory_deallocate(pointer);
        }
    }

    #[test]
    fn test_deallocate_null_pointer() {
        // Test that deallocating a null pointer doesn't crash
        Xila_memory_deallocate(core::ptr::null_mut());
    }

    #[test]
    fn test_reallocate_null_to_new() {
        unsafe {
            // Test reallocating from null (equivalent to allocation)
            let size = 128;
            let pointer = Xila_memory_reallocate(core::ptr::null_mut(), size);
            assert!(
                !pointer.is_null(),
                "Reallocation from null should work like allocation"
            );

            // Write to the memory to verify it's usable
            let ptr = pointer as *mut u8;
            for i in 0..size {
                *ptr.add(i) = (i % 256) as u8;
            }

            Xila_memory_deallocate(pointer);
        }
    }

    #[test]
    fn test_reallocate_expand() {
        unsafe {
            // Test expanding memory with reallocation
            let initial_size = 64;
            let expanded_size = 128;

            // First allocation
            let pointer = Xila_memory_reallocate(core::ptr::null_mut(), initial_size);
            assert!(!pointer.is_null(), "Initial allocation should succeed");

            // Fill with test data
            let ptr = pointer as *mut u8;
            for i in 0..initial_size {
                *ptr.add(i) = (i % 256) as u8;
            }

            // Expand the allocation
            let new_pointer = Xila_memory_reallocate(pointer, expanded_size);
            assert!(
                !new_pointer.is_null(),
                "Reallocation expansion should succeed"
            );

            // Verify original data is preserved
            let new_ptr = new_pointer as *mut u8;
            for i in 0..initial_size {
                assert_eq!(
                    *new_ptr.add(i),
                    (i % 256) as u8,
                    "Original data should be preserved"
                );
            }

            Xila_memory_deallocate(new_pointer);
        }
    }

    #[test]
    fn test_reallocate_shrink() {
        unsafe {
            // Test shrinking memory with reallocation
            let initial_size = 128;
            let shrunk_size = 64;

            // First allocation
            let pointer = Xila_memory_reallocate(core::ptr::null_mut(), initial_size);
            assert!(!pointer.is_null(), "Initial allocation should succeed");

            // Fill with test data
            let ptr = pointer as *mut u8;
            for i in 0..initial_size {
                *ptr.add(i) = (i % 256) as u8;
            }

            // Shrink the allocation
            let new_pointer = Xila_memory_reallocate(pointer, shrunk_size);
            assert!(
                !new_pointer.is_null(),
                "Reallocation shrinking should succeed"
            );

            // Verify data within new size is preserved
            let new_ptr = new_pointer as *mut u8;
            for i in 0..shrunk_size {
                assert_eq!(
                    *new_ptr.add(i),
                    (i % 256) as u8,
                    "Data within new size should be preserved"
                );
            }

            Xila_memory_deallocate(new_pointer);
        }
    }

    #[test]
    fn test_reallocate_to_zero() {
        unsafe {
            // Test reallocating to zero size (equivalent to deallocation)
            let initial_size = 64;

            let pointer = Xila_memory_reallocate(core::ptr::null_mut(), initial_size);
            assert!(!pointer.is_null(), "Initial allocation should succeed");

            let new_pointer = Xila_memory_reallocate(pointer, 0);
            // Zero-size reallocation might return null or a valid pointer
            // If it returns a valid pointer, we should deallocate it
            if !new_pointer.is_null() {
                Xila_memory_deallocate(new_pointer);
            }
        }
    }

    #[test]
    fn test_get_page_size() {
        let page_size = Xila_memory_get_page_size();

        // Page size should be a power of 2 and at least 4KB on most systems
        assert!(page_size > 0, "Page size should be greater than 0");
        assert!(page_size >= 4096, "Page size should be at least 4KB");
        assert!(
            page_size.is_power_of_two(),
            "Page size should be a power of 2"
        );
    }

    #[test]
    fn test_flush_data_cache() {
        // Test that flushing data cache doesn't crash
        Xila_memory_flush_data_cache();
    }

    #[test]
    fn test_flush_instruction_cache() {
        unsafe {
            // Test flushing instruction cache with valid memory
            let size = 128;
            let alignment = 8;
            let capabilities = Xila_memory_capabilities_execute;
            let hint_address = core::ptr::null_mut();

            let pointer = Xila_memory_allocate(hint_address, size, alignment, capabilities);
            assert!(
                !pointer.is_null(),
                "Executable memory allocation should succeed"
            );

            // Flush instruction cache for this memory region
            Xila_memory_flush_instruction_cache(pointer, size);

            Xila_memory_deallocate(pointer);
        }
    }

    #[test]
    fn test_multiple_allocations() {
        unsafe {
            let mut pointers = Vec::new();
            let allocation_count = 10;
            let size = 64;
            let alignment = 8;
            let capabilities = 0;

            // Allocate multiple memory blocks
            for _ in 0..allocation_count {
                let pointer =
                    Xila_memory_allocate(core::ptr::null_mut(), size, alignment, capabilities);
                assert!(!pointer.is_null(), "Each allocation should succeed");
                pointers.push(pointer);
            }

            // Verify each allocation is unique and writable
            for (i, &pointer) in pointers.iter().enumerate() {
                let ptr = pointer as *mut u8;
                let test_value = (i % 256) as u8;
                *ptr = test_value;
                assert_eq!(
                    *ptr, test_value,
                    "Each allocation should be independently writable"
                );
            }

            // Deallocate all memory blocks
            for &pointer in &pointers {
                Xila_memory_deallocate(pointer);
            }
        }
    }

    #[test]
    fn test_allocation_tracking() {
        unsafe {
            // Test that allocations are properly tracked for deallocation
            let size = 128;
            let alignment = 8;
            let capabilities = 0;

            let pointer =
                Xila_memory_allocate(core::ptr::null_mut(), size, alignment, capabilities);
            assert!(!pointer.is_null(), "Allocation should succeed");

            // The allocation should be tracked in the allocations table
            // We can't directly access the table, but deallocation should work
            Xila_memory_deallocate(pointer);

            // Double deallocation should be safe (should be handled gracefully)
            Xila_memory_deallocate(pointer);
        }
    }

    #[test]
    fn test_reallocation_tracking() {
        unsafe {
            // Test that reallocations properly update the tracking table
            let initial_size = 64;
            let new_size = 128;

            let pointer = Xila_memory_reallocate(core::ptr::null_mut(), initial_size);
            assert!(!pointer.is_null(), "Initial reallocation should succeed");

            let new_pointer = Xila_memory_reallocate(pointer, new_size);
            assert!(!new_pointer.is_null(), "Reallocation should succeed");

            // The new pointer should be properly tracked
            Xila_memory_deallocate(new_pointer);
        }
    }
}
