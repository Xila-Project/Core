use core::{alloc::GlobalAlloc, ptr::NonNull};

use crate::{CapabilitiesType, Layout, ManagerTrait};

unsafe extern "Rust" {
    unsafe static __XILA_MEMORY_ALLOCATOR: &'static dyn ManagerTrait;
}

/// A wrapper type that adapts any type implementing `Allocator_trait` to the standard
/// Rust `GlobalAlloc` trait.
///
/// This enables custom allocators that implement the Xila-specific `Allocator_trait`
/// to be used as the global memory allocator for Rust's allocation system.
pub struct Manager(&'static dyn ManagerTrait);

impl Manager {
    /// Creates a new instance of `Allocator_type` wrapping the provided allocator.
    ///
    /// # Parameters
    /// * `Allocator` - The allocator to wrap
    ///
    /// # Returns
    /// A new instance of `Allocator_type` containing the provided allocator.
    pub const fn new(allocator: &'static dyn ManagerTrait) -> Self {
        Self(allocator)
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
unsafe impl GlobalAlloc for Manager {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.0
            .allocate(CapabilitiesType::default(), Layout::from(layout))
            .map_or(core::ptr::null_mut(), |pointer| pointer.as_ptr())
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: core::alloc::Layout) {
        if !pointer.is_null() {
            self.0
                .deallocate(NonNull::new_unchecked(pointer), Layout::from(layout))
        }
    }
}

pub fn get_instance() -> &'static dyn ManagerTrait {
    unsafe { __XILA_MEMORY_ALLOCATOR }
}
