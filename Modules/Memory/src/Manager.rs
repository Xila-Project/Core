use core::{alloc::GlobalAlloc, ptr::NonNull};

use crate::{Capabilities_type, Layout_type, Manager_trait};

unsafe extern "Rust" {
    unsafe static __XILA_MEMORY_ALLOCATOR: &'static dyn Manager_trait;
}

/// A wrapper type that adapts any type implementing `Allocator_trait` to the standard
/// Rust `GlobalAlloc` trait.
///
/// This enables custom allocators that implement the Xila-specific `Allocator_trait`
/// to be used as the global memory allocator for Rust's allocation system.
pub struct Manager_type(&'static dyn Manager_trait);

impl Manager_type {
    /// Creates a new instance of `Allocator_type` wrapping the provided allocator.
    ///
    /// # Parameters
    /// * `Allocator` - The allocator to wrap
    ///
    /// # Returns
    /// A new instance of `Allocator_type` containing the provided allocator.
    pub const fn New(Allocator: &'static dyn Manager_trait) -> Self {
        Self(Allocator)
    }
}

/// Implementation of the standard library's `GlobalAlloc` trait for any wrapped
/// type that implements `Allocator_trait`.
///
/// This implementation delegates the standard allocation operations to the wrapped
/// allocator's methods, converting between Rust's and Xila's allocation types.
///
/// # Safety
/// The implementation upholds the safety guarantees required by `GlobalAlloc`:
/// - Memory is properly aligned according to the layout
/// - Deallocation uses the same layout that was used for allocation
unsafe impl GlobalAlloc for Manager_type {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.0
            .Allocate(Capabilities_type::default(), Layout_type::from(layout))
            .map_or(core::ptr::null_mut(), |pointer| pointer.as_ptr())
    }

    unsafe fn dealloc(&self, Pointer: *mut u8, Layout: core::alloc::Layout) {
        if !Pointer.is_null() {
            self.0
                .Deallocate(NonNull::new_unchecked(Pointer), Layout_type::from(Layout))
        }
    }
}

pub fn Get_instance() -> &'static dyn Manager_trait {
    unsafe { __XILA_MEMORY_ALLOCATOR }
}
