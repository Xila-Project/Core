use core::alloc::GlobalAlloc;

use crate::{Capabilities_type, Layout_type, Statistics_type};

pub trait Allocator_trait {
    unsafe fn Allocate(&self, Capabilities: Capabilities_type, Layout: Layout_type) -> *mut u8;
    unsafe fn Deallocate(&self, Pointer: *mut u8, Layout: Layout_type);
    unsafe fn Get_statistics(&self) -> Statistics_type;
}

pub struct Allocator_type<T: Allocator_trait>(T);

unsafe impl<T: Allocator_trait> GlobalAlloc for Allocator_type<T> {
    unsafe fn alloc(&self, Layout: core::alloc::Layout) -> *mut u8 {
        self.0
            .Allocate(Capabilities_type::default(), Layout_type::from(Layout))
    }

    unsafe fn dealloc(&self, Pointer: *mut u8, Layout: core::alloc::Layout) {
        self.0.Deallocate(Pointer, Layout_type::from(Layout))
    }
}

#[macro_export]
macro_rules! Instantiate_allocator {
    ($Allocator:expr) => {
        #[global_allocator]
        static ALLOCATOR: $crate::Allocator_type<_> = $crate::Allocator_type($Allocator);
    };
}
