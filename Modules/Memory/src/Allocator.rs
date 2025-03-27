use core::{alloc::GlobalAlloc, ptr::NonNull};

use crate::{Capabilities_type, Layout_type, Statistics_type};

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
pub trait Allocator_trait {
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

    unsafe fn Get_used(&self) -> usize;

    unsafe fn Get_free(&self) -> usize;
}

/// A wrapper type that adapts any type implementing `Allocator_trait` to the standard
/// Rust `GlobalAlloc` trait.
///
/// This enables custom allocators that implement the Xila-specific `Allocator_trait`
/// to be used as the global memory allocator for Rust's allocation system.
pub struct Allocator_type<T: Allocator_trait>(T);

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
unsafe impl<T: Allocator_trait> GlobalAlloc for Allocator_type<T> {
    unsafe fn alloc(&self, Layout: core::alloc::Layout) -> *mut u8 {
        self.0
            .Allocate(Capabilities_type::default(), Layout_type::from(Layout))
            .map_or(core::ptr::null_mut(), |Pointer| Pointer.as_ptr())
    }

    unsafe fn dealloc(&self, Pointer: *mut u8, Layout: core::alloc::Layout) {
        if !Pointer.is_null() {
            self.0
                .Deallocate(NonNull::new_unchecked(Pointer), Layout_type::from(Layout))
        }
    }
}

/// Macro to instantiate a global allocator using a Xila memory allocator.
///
/// This macro creates a static global allocator using the provided expression and
/// applies the `#[global_allocator]` attribute to it. This is the recommended way
/// to set up the global allocator in applications using Xila memory management.
///
/// # Example
/// ```rust,ignore
/// use Memory::{Instantiate_allocator};
///
/// struct My_allocator;
///
/// // Create a custom allocator instance
/// let Custom_allocator = My_allocator::new();
///
/// // Set it as the global allocator
/// Instantiate_allocator!(Custom_allocator);
/// ```
#[macro_export]
macro_rules! Instantiate_allocator {
    ($Allocator:expr) => {
        #[global_allocator]
        static ALLOCATOR: $crate::Allocator_type<_> = $crate::Allocator_type($Allocator);
    };
}
