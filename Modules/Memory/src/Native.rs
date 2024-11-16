use core::ptr::NonNull;
use std::os::raw::c_void;

use libc::{
    mmap, mprotect, munmap, sysconf, MAP_32BIT, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED, MAP_PRIVATE,
    PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE, _SC_PAGE_SIZE,
};

use crate::{Flags_type, Layout_type, Memory_allocator_trait, Protection_type};

// - Native platform

pub struct Memory_allocator_type;

impl Memory_allocator_type {
    pub const fn New() -> Self {
        Memory_allocator_type {}
    }
}

impl Memory_allocator_trait for Memory_allocator_type {
    unsafe fn Allocate_custom(
        &self,
        Hint_address: Option<NonNull<u8>>,
        Layout: Layout_type,
        Protection: Protection_type,
        Flags: Flags_type,
    ) -> Option<NonNull<u8>> {
        let mut Libc_flags = 0;

        if Flags.Get_private() {
            Libc_flags |= MAP_PRIVATE;
        }

        if Flags.Get_anonymous() {
            Libc_flags |= MAP_ANONYMOUS;
        }

        if Flags.Get_fixed() {
            Libc_flags |= MAP_FIXED;
        }

        if Flags.Get_address_32_bits() {
            Libc_flags |= MAP_32BIT;
        }

        // TODO : Add MAP_JIT flag for macOS, iOS and ARM64

        let Protection = Get_libc_protection(Protection);

        let Request_size = Round_page_size(Layout.Get_size(), self.Get_page_size());

        if Request_size < Layout.Get_size() {
            // Overflow
            return None;
        }

        let Pointer = mmap(
            Hint_address.map_or(std::ptr::null_mut(), |Address| {
                Address.as_ptr() as *mut c_void
            }),
            Request_size,
            Protection,
            Libc_flags,
            -1,
            0,
        );

        if Pointer == MAP_FAILED {
            None
        } else {
            Some(NonNull::new_unchecked(Pointer as *mut u8))
        }
    }

    unsafe fn Deallocate_custom(&self, Address: NonNull<u8>, Length: usize) -> bool {
        let Request_size = Round_page_size(Length, self.Get_page_size());

        if Request_size < Length {
            // Overflow
            return false;
        }

        munmap(Address.as_ptr() as *mut c_void, Request_size) == 0
    }

    unsafe fn Protect(
        &self,
        Address: NonNull<u8>,
        Length: usize,
        Protection: Protection_type,
    ) -> bool {
        let Protection = Get_libc_protection(Protection);

        let Request_size = Round_page_size(Length, self.Get_page_size());

        mprotect(Address.as_ptr() as *mut c_void, Request_size, Protection) == 0
    }

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
