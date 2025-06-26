use core::ptr::NonNull;

use crate::{Capabilities_type, Layout_type};

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

    /// Reallocates memory with a new layout.
    ///
    /// This method changes the size or alignment of a previously allocated memory block.
    /// If the pointer is `None`, this behaves like a normal allocation.
    /// If reallocation is successful, the contents of the old memory are preserved
    /// up to the minimum of the old and new sizes.
    ///
    /// # Parameters
    /// * `Pointer` - Optional pointer to the memory to reallocate. If `None`, acts as a new allocation
    /// * `Old_layout` - The layout that was used for the original allocation
    /// * `New_layout` - The new layout requirements for the memory
    ///
    /// # Returns
    /// A pointer to the reallocated memory with the new layout, or `None` if reallocation failed.
    /// The protection is set to [`crate::Protection_type::Read_write`].
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that:
    /// - If `Pointer` is `Some`, it was allocated by this allocator
    /// - The `Old_layout` matches the one used for the original allocation
    /// - The old memory is not used after successful reallocation
    /// - The returned memory is properly initialized before use
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

    /// Flushes the instruction cache for a specific memory region.
    ///
    /// This method ensures that any cached instructions in the specified memory
    /// region are synchronized with main memory. This is particularly important
    /// on architectures with separate instruction and data caches when code
    /// has been modified at runtime.
    ///
    /// # Parameters
    /// * `Address` - Pointer to the start of the memory region to flush
    /// * `Size` - Size in bytes of the memory region to flush
    ///
    /// # Note
    /// The default implementation does nothing and can be overridden by specific
    /// allocators that need to handle instruction cache management.
    fn Flush_instruction_cache(&self, _Address: NonNull<u8>, _Size: usize) {
        // Default implementation does nothing, can be overridden by specific allocators
    }

    /// Flushes the data cache to ensure memory coherency.
    ///
    /// This method ensures that any cached data is written back to main memory
    /// and that the cache is synchronized. This is important for maintaining
    /// memory coherency, especially in multi-core systems or when dealing with
    /// memory-mapped I/O operations.
    ///
    /// # Note
    /// The default implementation does nothing and can be overridden by specific
    /// allocators that need to handle data cache management.
    fn Flush_data_cache(&self) {
        // Default implementation does nothing, can be overridden by specific allocators
    }

    /// Returns the page size used by this memory allocator.
    ///
    /// The page size is the smallest unit of memory that can be allocated
    /// by the underlying memory management system. This information is useful
    /// for optimizing memory allocation patterns and understanding alignment
    /// requirements.
    ///
    /// # Returns
    /// The page size in bytes used by this allocator.
    ///
    /// # Note
    /// The default implementation returns 4096 bytes (4 KiB), which is a common
    /// page size on many architectures. Specific allocators can override this
    /// to return the actual page size of their underlying memory management system.
    fn Get_page_size(&self) -> usize {
        // Default implementation returns a common page size, can be overridden by specific allocators
        4096 // 4 KiB is a common page size
    }
}
