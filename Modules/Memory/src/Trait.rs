use core::{alloc::GlobalAlloc, ptr::NonNull};

use crate::{Capabilities_type, Layout_type, Protection_type};

/// Trait that defines the interface for memory allocators in the Xila system.
///
/// Implementors of this trait can be used to manage memory allocation and deallocation
/// operations with specific capabilities. The trait provides the foundation for
/// custom memory allocation strategies.
///
/// # Safety
///
/// All methods in this trait are unsafe because they deal with raw memory and can
/// cause undefined behavior if used incorrectly, such as:
/// - Deallocating memory that wasn't allocated with the same allocator
/// - Using memory after it has been deallocated
/// - Deallocating the same memory multiple times
pub trait Manager_trait: Send + Sync {
    /// Allocates memory with the specified capabilities and layout.
    ///
    /// # Parameters
    /// * `Capabilities` - Specific requirements for the allocation
    /// * `Layout` - Size and alignment requirements for the allocation
    ///
    /// # Returns
    /// A pointer to the allocated memory, where the protection is set to [`crate::Protection_type::Read_write`], or a null pointer if allocation failed.
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that:
    /// - The returned memory is properly initialized before use
    /// - The memory is properly deallocated when no longer needed
    unsafe fn Allocate(
        &self,
        Capabilities: Capabilities_type,
        Layout: Layout_type,
    ) -> Option<NonNull<u8>>;

    /// Deallocates memory previously allocated by this allocator.
    ///
    /// # Parameters
    /// * `Pointer` - Pointer to the memory to deallocate
    /// * `Layout` - The layout that was used to allocate the memory
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that:
    /// - The pointer was allocated by this allocator
    /// - The layout matches the one used for allocation
    /// - The memory is not used after deallocation
    /// - The memory is not deallocated multiple times
    unsafe fn Deallocate(&self, Pointer: NonNull<u8>, Layout: Layout_type);

    unsafe fn Reallocate(
        &self,
        Pointer: Option<NonNull<u8>>,
        Old_layout: Layout_type,
        New_layout: Layout_type,
    ) -> Option<NonNull<u8>> {
        // Default implementation simply deallocates and allocates again
        let Memory = self.Allocate(Capabilities_type::default(), New_layout)?;

        // Copy the old data to the new memory
        let Pointer = match Pointer {
            Some(ptr) => ptr,
            None => return Some(Memory),
        };

        let Old_size = Old_layout.size();
        let New_size = New_layout.size();
        if Old_size > 0 && New_size > 0 {
            let Old_ptr = Pointer.as_ptr();
            let New_ptr = Memory.as_ptr();
            core::ptr::copy_nonoverlapping(Old_ptr, New_ptr, core::cmp::min(Old_size, New_size));
        }
        // Deallocate the old memory
        self.Deallocate(Pointer, Old_layout);

        Some(Memory)
    }

    /// Returns the amount of memory currently used in this allocator.
    ///
    /// # Returns
    /// The number of bytes currently allocated.
    ///
    /// # Safety
    /// This function is unsafe because it may rely on internal allocator state
    /// that could be concurrently modified by other threads.
    unsafe fn Get_used(&self) -> usize;

    /// Returns the amount of memory currently available in this allocator.
    ///
    /// # Returns
    /// The number of bytes available for allocation.
    ///
    /// # Safety
    /// This function is unsafe because it may rely on internal allocator state
    /// that could be concurrently modified by other threads.
    unsafe fn Get_free(&self) -> usize;

    fn Flush_instruction_cache(&self, _Adresss: NonNull<u8>, _Size: usize) {
        // Default implementation does nothing, can be overridden by specific allocators
    }

    fn Flush_data_cache(&self) {
        // Default implementation does nothing, can be overridden by specific allocators
    }

    fn Get_page_size(&self) -> usize {
        // Default implementation returns a common page size, can be overridden by specific allocators
        4096 // 4 KiB is a common page size
    }
}
