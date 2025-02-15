use std::ptr::NonNull;

use crate::{Flags_type, Layout_type, Protection_type};

pub trait Memory_allocator_trait {
    /// Allocate a memory region.
    /// # Safety
    /// This function is unsafe because it allocates a memory region and returns a pointer to it which can lead to undefined behavior.
    unsafe fn Allocate_custom(
        &self,
        Hint_address: Option<NonNull<u8>>,
        Layout: Layout_type,
        Protection: Protection_type,
        Flags: Flags_type,
    ) -> Option<NonNull<u8>>;

    /// Deallocate a memory region.
    /// # Safety
    /// This function is unsafe because it deallocates a memory region which can lead to undefined behavior.
    unsafe fn Deallocate_custom(&self, Address: NonNull<u8>, Length: usize) -> bool;

    /// Change the protection of a memory region.
    /// # Safety
    /// This function is unsafe because it changes the protection of a memory region which can lead to undefined behavior.
    unsafe fn Protect(
        &self,
        Address: NonNull<u8>,
        Length: usize,
        Protection: Protection_type,
    ) -> bool;

    /// Get the page size of the system.
    /// # Safety
    /// This function is unsafe because it returns the page size of the system which can lead to undefined behavior.
    fn Get_page_size(&self) -> usize;

    fn Flush_data_cache(&self) {}

    fn Flush_instruction_cache(&self, _Address: NonNull<u8>, _Size: usize) {}
}
