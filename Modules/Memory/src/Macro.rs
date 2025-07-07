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
macro_rules! Instantiate_global_allocator {
    ($Allocator:ty) => {
        static A: $Allocator = <$Allocator>::new();

        #[global_allocator]
        #[no_mangle]
        pub static __XILA_MEMORY_ALLOCATOR: $crate::Manager_type = $crate::Manager_type::New(&A);
    };
}
