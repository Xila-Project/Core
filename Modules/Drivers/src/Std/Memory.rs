use core::ptr::NonNull;
use std::{alloc, os::raw::c_void};

use libc::{
    mmap, mprotect, munmap, sysconf, MAP_32BIT, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED, MAP_PRIVATE,
    PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE, _SC_PAGE_SIZE,
};
use File_system::Statistics_type;
use Memory::{Allocator_trait, Capabilities_type, Layout_type, Protection_trait, Protection_type};

use crate::{Flags_type, Layout_type, Memory_allocator_trait, Protection_type};

// - Native platform

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Memory_manager_type;

impl Memory_manager_type {
    pub const fn New() -> Self {
        Memory_manager_type {}
    }
}

impl Allocator_trait for Memory_manager_type {
    unsafe fn Allocate(&self, _: Capabilities_type, Layout: Layout_type) -> Option<NonNull<u8>> {
        let Allocation = alloc::alloc(Layout);

        if Allocation.is_null() {
            None
        } else {
            Some(NonNull::new_unchecked(Allocation))
        }
    }

    unsafe fn Deallocate(&self, Pointer: NonNull<u8>, Layout: Layout_type) {
        alloc::dealloc(Pointer.as_ptr(), Layout)
    }

    unsafe fn Get_statistics(&self) -> Statistics_type {
        todo!()
    }
}

impl Protection_trait for Memory_manager_type {
    unsafe fn Set_protection(
        &self,
        Address: *mut u8,
        Size: usize,
        Protection: Protection_type,
    ) -> bool {
        let Protection = Get_libc_protection(Protection);

        let Request_size = Round_page_size(Length, self.Get_page_size());

        mprotect(Address.as_ptr() as *mut c_void, Request_size, Protection) == 0
    }
}

impl Memory_manager_type {
    fn Get_page_size(&self) -> usize {
        unsafe { sysconf(_SC_PAGE_SIZE) as usize }
    }
}

const fn Round_page_size(Size: usize, Page_size: usize) -> usize {
    (Size + Page_size - 1) & !(Page_size - 1) // Round up to the nearest page size
}

const fn Get_libc_protection(Protection: crate::Protection_type) -> i32 {
    let mut Libc_protection = PROT_NONE;

    if Protection.Get_execute() {
        Libc_protection |= PROT_EXEC;
    }
    if Protection.Get_read() {
        Libc_protection |= PROT_READ;
    }
    if Protection.Get_write() {
        Libc_protection |= PROT_WRITE;
    }

    Libc_protection
}
