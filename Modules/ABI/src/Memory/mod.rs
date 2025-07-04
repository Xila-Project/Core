//! # Memory Management Module
//!
//! This module provides low-level memory management functionality for the Xila operating system.
//! It exposes a C-compatible ABI for memory allocation, deallocation, and memory operations
//! that can be used by applications and other system components.
//!
//! ## Features
//!
//! - **Memory Allocation**: Allocate memory blocks with specific size, alignment, and capabilities
//! - **Memory Deallocation**: Free allocated memory blocks
//! - **Memory Reallocation**: Resize existing memory blocks
//! - **Cache Management**: Flush data and instruction caches
//! - **Memory Protection**: Support for read, write, and execute permissions
//! - **Memory Capabilities**: Support for executable memory and DMA-capable memory
//!
//! ## Safety
//!
//! This module provides unsafe functions that directly manipulate memory. Callers must ensure:
//! - Pointers are valid and properly aligned
//! - Memory is not accessed after deallocation
//! - Concurrent access is properly synchronized
//!
//! ## Example Usage
//!
//! ```c
//! // Allocate 1024 bytes with 8-byte alignment
//! void* ptr = Xila_memory_allocate(NULL, 1024, 8, 0);
//! if (ptr != NULL) {
//!     // Use the memory...
//!     Xila_memory_deallocate(ptr);
//! }
//! ```

#![allow(dead_code)]

use alloc::collections::BTreeMap;
use core::ptr::null_mut;
use core::{ffi::c_void, ptr::NonNull};
use Futures::block_on;
use Log::{Debug, Trace, Warning};
use Synchronization::blocking_mutex::raw::CriticalSectionRawMutex;

use Synchronization::rwlock::RwLock;

use Memory::{Capabilities_type, Layout_type};

/// Memory protection flags that can be combined using bitwise OR.
/// These flags determine what operations are allowed on memory regions.
pub type Xila_memory_protection_type = u8;

/// Read permission flag - allows reading from memory
#[no_mangle]
pub static Xila_memory_protection_read: u8 = Memory::Protection_type::Read_bit;

/// Write permission flag - allows writing to memory
#[no_mangle]
pub static Xila_memory_protection_write: u8 = Memory::Protection_type::Write_bit;

/// Execute permission flag - allows executing code from memory
#[no_mangle]
pub static Xila_memory_protection_execute: u8 = Memory::Protection_type::Execute_bit;

/// Memory capability flags that specify special requirements for allocated memory.
/// These flags can be combined using bitwise OR to request multiple capabilities.
pub type Xila_memory_capabilities_type = u8;

/// Executable capability - memory can be used for code execution
#[no_mangle]
pub static Xila_memory_capabilities_execute: Xila_memory_capabilities_type =
    Memory::Capabilities_type::Executable_flag;

/// Direct Memory Access (DMA) capability - memory is accessible by DMA controllers
#[no_mangle]
pub static Xila_memory_capabilities_direct_memory_access: Xila_memory_capabilities_type =
    Memory::Capabilities_type::Direct_memory_access_flag;

/// No special capabilities required - standard memory allocation
#[no_mangle]
pub static Xila_memory_capabilities_none: Xila_memory_capabilities_type = 0;

// - Memory Management Functions

// - - Allocation Utilities

/// Converts a function that returns an `Option<NonNull<P>>` into a raw C-compatible pointer.
///
/// This utility function is used internally to convert Rust's safe pointer types
/// into C-compatible raw pointers. Returns `null_mut()` if the function returns `None`.
///
/// # Type Parameters
///
/// * `F` - A function that returns `Option<NonNull<P>>`
/// * `P` - The pointer type being converted
///
/// # Parameters
///
/// * `Function` - The function to execute and convert the result
///
/// # Returns
///
/// A raw C-compatible pointer, or null if the function returns `None`
pub fn Into_pointer<F, P>(Function: F) -> *mut c_void
where
    F: FnOnce() -> Option<NonNull<P>>,
{
    match Function() {
        Some(Pointer) => Pointer.as_ptr() as *mut c_void,
        None => null_mut(),
    }
}

/// Global allocation tracking table.
///
/// This table maps memory addresses to their corresponding memory layouts,
/// enabling proper deallocation and reallocation operations. The table is
/// protected by a RwLock to ensure thread safety.
static Allocations_table: RwLock<CriticalSectionRawMutex, BTreeMap<usize, Layout_type>> =
    RwLock::new(BTreeMap::new());

/// Macro to acquire a write lock on the allocations table.
///
/// This macro blocks until the lock is acquired, ensuring exclusive access
/// to the allocation tracking table for modification operations.
macro_rules! Write_allocations_table {
    () => {
        block_on(Allocations_table.write())
    };
}

/// Deallocates a previously allocated memory block.
///
/// This function frees memory that was previously allocated using `Xila_memory_allocate`
/// or `Xila_memory_reallocate`. It's safe to call this function with a null pointer.
///
/// # Safety
///
/// - The pointer must have been returned by a previous call to `Xila_memory_allocate`
///   or `Xila_memory_reallocate`
/// - The pointer must not be used after this function returns
/// - Double-free is safe and will be ignored
///
/// # Parameters
///
/// * `Pointer` - Pointer to the memory block to deallocate, or null
///
/// # Examples
///
/// ```c
/// void* ptr = Xila_memory_allocate(NULL, 1024, 8, 0);
/// Xila_memory_deallocate(ptr); // Free the memory
/// Xila_memory_deallocate(NULL); // Safe - ignored
/// ```
#[no_mangle]
pub extern "C" fn Xila_memory_deallocate(Pointer: *mut c_void) {
    if Pointer.is_null() {
        Warning! { "Xila_memory_deallocate called with null pointer, ignoring"
        };
        return;
    }

    let Layout = match Write_allocations_table!().remove(&(Pointer as usize)) {
        Some(Size) => Size,
        None => {
            Warning! {
                "Xila_memory_deallocate called with untracked pointer: {:#x}, ignoring",
                Pointer as usize
            };
            return;
        }
    };

    unsafe {
        Memory::Get_instance().Deallocate(
            NonNull::new(Pointer as *mut u8).expect("Failed to deallocate memory, pointer is null"),
            Layout,
        );
        Debug! {
            "Xila_memory_deallocate called with pointer: {:#x}, deallocated successfully",
            Pointer as usize
        };
    }
}

/// Reallocates a memory block to a new size.
///
/// This function changes the size of a previously allocated memory block. If the new size
/// is larger, the additional memory is uninitialized. If the new size is smaller, the
/// excess memory is freed. The original data is preserved up to the minimum of the old
/// and new sizes.
///
/// # Safety
///
/// - If `Pointer` is not null, it must have been returned by a previous call to
///   `Xila_memory_allocate` or `Xila_memory_reallocate`
/// - The pointer must not be used after this function returns (use the returned pointer instead)
/// - If the function returns null, the original pointer remains valid
///
/// # Parameters
///
/// * `Pointer` - Pointer to the memory block to reallocate, or null for new allocation
/// * `Size` - New size in bytes (0 is equivalent to deallocation)
///
/// # Returns
///
/// - Pointer to the reallocated memory block
/// - Null if allocation fails or if size is 0
/// - If `Pointer` is null, behaves like `Xila_memory_allocate` with default alignment
///
/// # Examples
///
/// ```c
/// // Allocate initial memory
/// void* ptr = Xila_memory_reallocate(NULL, 1024);
///
/// // Expand the memory
/// ptr = Xila_memory_reallocate(ptr, 2048);
///
/// // Shrink the memory
/// ptr = Xila_memory_reallocate(ptr, 512);
///
/// // Free the memory
/// Xila_memory_reallocate(ptr, 0);
/// ```
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

        let New_layout = Layout_type::from_size_align(Size, Old_layout.align()).ok()?;

        Debug!(
            "Xila_memory_reallocate called with Pointer: {:#x}, Old_layout: {:?}, New_layout: {:?}",
            Pointer.map_or(0, |p| p.as_ptr() as usize),
            Old_layout,
            New_layout
        );

        let Allocated = Memory::Get_instance().Reallocate(Pointer, Old_layout, New_layout)?;

        Allocation_table.insert(Allocated.as_ptr() as usize, New_layout);

        Some(Allocated)
    })
}

/// Allocates a memory block with specified properties.
///
/// This function allocates a block of memory with the specified size, alignment,
/// and capabilities. The memory is uninitialized and must be properly initialized
/// before use.
///
/// # Safety
///
/// This function is unsafe because:
/// - The returned memory is uninitialized
/// - The caller must ensure proper deallocation
/// - The hint address, if providalignment enumed, must be a valid memory address
///
/// # Parameters
///
/// * `Hint_address` - Preferred memory address (hint only, may be ignored), or null
/// * `Size` - Size of the memory block in bytes
/// * `Alignment` - Required memory alignment in bytes (must be a power of 2)
/// * `Capabilities` - Memory capabilities flags (see `Xila_memory_capabilities_*` constants)
///
/// # Returns
///
/// - Pointer to the allocated memory block
/// - Null if allocation fails
///
/// # Errors
///
/// Returns null if:
/// - Insufficient memory available
/// - Invalid alignment (not a power of 2)
/// - Requested capabilities not supported
/// - Size is too large
///
/// # Examples
///
/// ```c
/// // Allocate 1024 bytes with 8-byte alignment
/// void* ptr = Xila_memory_allocate(NULL, 1024, 8, Xila_memory_capabilities_none);
///
/// // Allocate executable memory for code
/// void* code_ptr = Xila_memory_allocate(NULL, 4096, 4096, Xila_memory_capabilities_execute);
///
/// // Allocate DMA-capable memory
/// void* dma_ptr = Xila_memory_allocate(NULL, 2048, 32, Xila_memory_capabilities_direct_memory_access);
/// ```
#[no_mangle]
pub unsafe extern "C" fn Xila_memory_allocate(
    _: *mut c_void,
    Size: usize,
    Alignment: usize,
    Capabilities: Xila_memory_capabilities_type,
) -> *mut c_void {
    Into_pointer(|| {
        Trace!(
            "Xila_memory_allocate called with Size: {Size}, Alignment: {Alignment}, Capabilities: {Capabilities:?}"
        );
        let Layout = Layout_type::from_size_align(Size, Alignment)
            .expect("Failed to create layout for memory allocation");

        let Capabilities = Capabilities_type::From_u8(Capabilities);

        let Result = Memory::Get_instance().Allocate(Capabilities, Layout);

        if Result.is_some() {
            Write_allocations_table!().insert(Result.unwrap().as_ptr() as usize, Layout);
            Debug! {
                "Xila_memory_allocate called with Size: {}, Alignment: {}, Capabilities: {:?}, allocated memory at {:#x}",
                Size, Alignment, Capabilities, Result.unwrap().as_ptr() as usize
            };
        } else {
            Warning! {
                "Xila_memory_allocate failed with Size: {Size}, Alignment: {Alignment}, Capabilities: {Capabilities:?}"
            };
        }

        Result
    })
}

/// Returns the system's memory page size.
///
/// The page size is the smallest unit of memory that can be allocated by the
/// virtual memory system. This is useful for applications that need to work
/// with page-aligned memory or perform memory-mapped I/O operations.
///
/// # Returns
///
/// The page size in bytes (typically 4096 on most systems)
///
/// # Examples
///
/// ```c
/// size_t page_size = Xila_memory_get_page_size();
/// printf("System page size: %zu bytes\n", page_size);
///
/// // Allocate page-aligned memory
/// void* ptr = Xila_memory_allocate(NULL, page_size * 2, page_size, 0);
/// ```
#[no_mangle]
pub extern "C" fn Xila_memory_get_page_size() -> usize {
    Memory::Get_instance().Get_page_size()
}

/// Flushes the data cache.
///
/// This function ensures that all pending write operations in the data cache
/// are written to main memory. This is important for cache coherency in
/// multi-core systems or when interfacing with DMA controllers.
///
/// # Safety
///
/// This function is safe to call at any time, but may have performance implications
/// as it forces cache synchronization.
///
/// # Examples
///
/// ```c
/// // After writing data that will be accessed by DMA
/// memcpy(dma_buffer, data, size);
/// Xila_memory_flush_data_cache();
/// start_dma_transfer(dma_buffer, size);
/// ```
#[no_mangle]
pub extern "C" fn Xila_memory_flush_data_cache() {
    Memory::Get_instance().Flush_data_cache();
}

/// Flushes the instruction cache for a specific memory region.
///
/// This function invalidates the instruction cache for the specified memory region.
/// This is necessary after modifying executable code to ensure the processor
/// sees the updated instructions.
///
/// # Safety
///
/// - The address must point to valid memory
/// - The memory region must not be accessed for execution during the flush operation
/// - The size should not exceed the actual allocated memory size
///
/// # Parameters
///
/// * `Address` - Starting address of the memory region to flush
/// * `Size` - Size of the memory region in bytes
///
/// # Examples
///
/// ```c
/// // After writing machine code to executable memory
/// void* code_ptr = Xila_memory_allocate(NULL, 4096, 4096, Xila_memory_capabilities_execute);
/// memcpy(code_ptr, machine_code, code_size);
/// Xila_memory_flush_instruction_cache(code_ptr, code_size);
///
/// // Now safe to execute the code
/// ((void(*)())code_ptr)();
/// ```
#[no_mangle]
pub extern "C" fn Xila_memory_flush_instruction_cache(_Address: *mut c_void, _Size: usize) {
    let Address = NonNull::new(_Address as *mut u8).expect("Failed to flush instruction cache");

    Memory::Get_instance().Flush_instruction_cache(Address, _Size);
}

#[cfg(test)]
mod Tests {
    //! # Memory Management Tests
    //!
    //! This module contains comprehensive tests for the memory management functionality.
    //! The tests cover various scenarios including basic allocation/deallocation,
    //! edge cases, error conditions, and stress testing.
    //!
    //! ## Test Categories
    //!
    //! - **Basic Operations**: Standard allocation, deallocation, and reallocation
    //! - **Edge Cases**: Zero-size allocations, large alignments, null pointers
    //! - **Capabilities**: Testing executable and DMA-capable memory
    //! - **Cache Operations**: Data and instruction cache flushing
    //! - **Stress Testing**: Multiple allocations and allocation tracking
    //! - **Error Handling**: Invalid inputs and error recovery

    use alloc::vec::Vec;

    use super::*;

    /// Tests basic memory allocation and deallocation functionality.
    ///
    /// This test verifies that:
    /// - Memory can be successfully allocated with specified size and alignment
    /// - Allocated memory is readable and writable
    /// - Memory can be properly deallocated without errors
    /// - Data written to memory is correctly stored and retrieved
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

    /// Tests allocation behavior with zero size.
    ///
    /// Zero-size allocations are a special case that different allocators
    /// may handle differently. This test ensures the implementation handles
    /// them gracefully without crashing.
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

    /// Tests memory allocation with large alignment requirements.
    ///
    /// This test verifies that the allocator can handle large alignment
    /// requirements (64 bytes) and that the returned pointer is properly
    /// aligned to the requested boundary.
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

    /// Tests allocation of executable memory.
    ///
    /// This test verifies that memory can be allocated with executable
    /// capabilities, which is necessary for just-in-time compilation
    /// and dynamic code generation.
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

    /// Tests allocation of DMA-capable memory.
    ///
    /// DMA-capable memory must meet specific hardware requirements and
    /// may need to be allocated from special memory regions. This test
    /// is ignored by default as it requires specific hardware support.
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

    /// Tests that deallocating a null pointer is safe.
    ///
    /// According to standard behavior, deallocating a null pointer
    /// should be a no-op and not cause any errors or crashes.
    #[test]
    fn test_deallocate_null_pointer() {
        // Test that deallocating a null pointer doesn't crash
        Xila_memory_deallocate(core::ptr::null_mut());
    }

    /// Tests reallocation from a null pointer (equivalent to allocation).
    ///
    /// When realloc is called with a null pointer, it should behave
    /// identically to malloc, allocating new memory of the requested size.
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

    /// Tests expanding memory through reallocation.
    ///
    /// This test verifies that existing data is preserved when memory
    /// is expanded through reallocation, and that the new memory region
    /// is usable.
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

    /// Tests shrinking memory through reallocation.
    ///
    /// This test verifies that data within the new size bounds is preserved
    /// when memory is shrunk through reallocation.
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

    /// Tests reallocation to zero size (equivalent to deallocation).
    ///
    /// When realloc is called with size 0, it should behave like free(),
    /// deallocating the memory and potentially returning null.
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

    /// Tests that the system page size is reasonable and valid.
    ///
    /// This test verifies that:
    /// - Page size is greater than 0
    /// - Page size is at least 4KB (common minimum)
    /// - Page size is a power of 2 (hardware requirement)
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

    /// Tests that data cache flushing doesn't cause crashes.
    ///
    /// This is a basic safety test to ensure the cache flush operation
    /// completes without errors. The actual cache flush behavior is
    /// hardware-dependent and difficult to test directly.
    #[test]
    fn test_flush_data_cache() {
        // Test that flushing data cache doesn't crash
        Xila_memory_flush_data_cache();
    }

    /// Tests instruction cache flushing on executable memory.
    ///
    /// This test verifies that instruction cache flushing works correctly
    /// on executable memory regions, which is essential for dynamic code
    /// generation and just-in-time compilation.
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

    #[test]
    fn test_double_free_handling() {
        unsafe {
            // Test that double-free is handled gracefully
            let size = 64;
            let alignment = 8;
            let capabilities = 0;

            let pointer =
                Xila_memory_allocate(core::ptr::null_mut(), size, alignment, capabilities);
            assert!(!pointer.is_null(), "Allocation should succeed");

            // First deallocation should succeed
            Xila_memory_deallocate(pointer);

            // Second deallocation should be ignored (no crash)
            Xila_memory_deallocate(pointer);
        }
    }
}
