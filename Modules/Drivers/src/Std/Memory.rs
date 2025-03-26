use core::ptr::NonNull;
use std::{alloc::Layout, os::raw::c_void};

use libc::{
    mmap, mprotect, mremap, munmap, sysconf, MAP_32BIT, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED,
    MAP_PRIVATE, MREMAP_MAYMOVE, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE, _SC_PAGE_SIZE,
};
use linked_list_allocator::Heap;
use File_system::Statistics_type;
use Memory::{Allocator_trait, Capabilities_type, Layout_type, Protection_trait, Protection_type};

use crate::{Flags_type, Memory_allocator_trait, Protection_type};

// Initial heap size and growth constants
const INITIAL_HEAP_SIZE: usize = 1024 * 1024; // 1MB
const HEAP_GROWTH_FACTOR: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Memory_manager_type {
    Executable: Heap,
    Executable_slice: &'static mut [u8],
    Non_executable: Heap,
    Non_executable_slice: &'static mut [u8],
}

impl Memory_manager_type {
    pub const fn New() -> Self {
        Memory_manager_type {
            Executable: Heap::empty(),
            Executable_slice: &mut [],
            Non_executable: Heap::empty(),
            Non_executable_slice: &mut [],
        }
    }

    pub unsafe fn Initialize(&mut self) {
        // Initialize executable heap
        let executable_protection = Protection_type::new_with_execute(true, true, true);
        self.Executable_slice = Self::allocate_memory(INITIAL_HEAP_SIZE, executable_protection);
        self.Executable.init(
            self.Executable_slice.as_mut_ptr() as usize,
            INITIAL_HEAP_SIZE,
        );

        // Initialize non-executable heap
        let non_executable_protection = Protection_type::new_with_execute(false, true, true);
        self.Non_executable_slice =
            Self::allocate_memory(INITIAL_HEAP_SIZE, non_executable_protection);
        self.Non_executable.init(
            self.Non_executable_slice.as_mut_ptr() as usize,
            INITIAL_HEAP_SIZE,
        );
    }

    unsafe fn allocate_memory(size: usize, protection: Protection_type) -> &'static mut [u8] {
        let page_size = sysconf(_SC_PAGE_SIZE) as usize;
        let size = Self::round_page_size(size, page_size);

        let ptr = mmap(
            std::ptr::null_mut(),
            size,
            Self::get_libc_protection(protection),
            MAP_PRIVATE | MAP_ANONYMOUS | MAP_32BIT,
            -1,
            0,
        );

        if ptr == MAP_FAILED {
            panic!("Failed to allocate memory");
        }

        std::slice::from_raw_parts_mut(ptr as *mut u8, size)
    }

    fn get_page_size(&self) -> usize {
        unsafe { sysconf(_SC_PAGE_SIZE) as usize }
    }

    const fn round_page_size(size: usize, page_size: usize) -> usize {
        (size + page_size - 1) & !(page_size - 1) // Round up to the nearest page size
    }

    unsafe fn expand_heap(&mut self, needs_execute: bool, required_size: usize) -> bool {
        let (heap, slice, protection) = if needs_execute {
            (
                &mut self.Executable,
                &mut self.Executable_slice,
                Protection_type::new_with_execute(true, true, true),
            )
        } else {
            (
                &mut self.Non_executable,
                &mut self.Non_executable_slice,
                Protection_type::new_with_execute(false, true, true),
            )
        };

        // Calculate new size (double current size or add enough for required allocation)
        let current_size = slice.len();
        let new_size = current_size.max(required_size) * HEAP_GROWTH_FACTOR;
        let new_size = Self::round_page_size(new_size, self.get_page_size());

        // Use mremap to expand the memory
        let new_ptr = mremap(
            slice.as_mut_ptr() as *mut c_void,
            current_size,
            new_size,
            MREMAP_MAYMOVE,
            std::ptr::null_mut(),
        );

        if new_ptr == MAP_FAILED {
            return false;
        }

        // Update the slice
        *slice = std::slice::from_raw_parts_mut(new_ptr as *mut u8, new_size);

        // Reinitialize the heap with the new memory
        let used_bytes = heap.used();
        let free_bytes = heap.free();

        // Only extend the heap with the additional memory
        if used_bytes + free_bytes < new_size {
            let additional_start = (new_ptr as usize) + used_bytes + free_bytes;
            let additional_size = new_size - (used_bytes + free_bytes);
            heap.extend(additional_start, additional_size);
        }

        true
    }
}

impl Allocator_trait for Memory_manager_type {
    unsafe fn Allocate(
        &mut self,
        capabilities: Capabilities_type,
        layout: Layout_type,
    ) -> Option<NonNull<u8>> {
        let needs_execute = capabilities.Get_execute_capability();
        let size = layout.Get_size();
        let align = layout.Get_align();

        let (heap, needs_expansion) = if needs_execute {
            let allocated = self.Executable.allocate_first_fit(size, align);
            (&mut self.Executable, allocated.is_err())
        } else {
            let allocated = self.Non_executable.allocate_first_fit(size, align);
            (&mut self.Non_executable, allocated.is_err())
        };

        // If allocation failed, try to expand the heap
        if needs_expansion {
            if !self.expand_heap(needs_execute, size) {
                return None;
            }

            // Retry allocation after expansion
            let result = if needs_execute {
                self.Executable.allocate_first_fit(size, align)
            } else {
                self.Non_executable.allocate_first_fit(size, align)
            };

            match result {
                Ok(ptr) => Some(NonNull::new_unchecked(ptr as *mut u8)),
                Err(_) => None,
            }
        } else {
            // Return the successful allocation
            let result = if needs_execute {
                self.Executable.allocate_first_fit(size, align)
            } else {
                self.Non_executable.allocate_first_fit(size, align)
            };

            match result {
                Ok(ptr) => Some(NonNull::new_unchecked(ptr as *mut u8)),
                Err(_) => None,
            }
        }
    }

    unsafe fn Deallocate(&mut self, pointer: NonNull<u8>, layout: Layout_type) {
        let ptr = pointer.as_ptr() as usize;
        let size = layout.Get_size();
        let align = layout.Get_align();

        // Check which heap contains this pointer
        let exec_start = self.Executable_slice.as_ptr() as usize;
        let exec_end = exec_start + self.Executable_slice.len();

        if (exec_start..exec_end).contains(&ptr) {
            self.Executable.deallocate(ptr, size, align);
        } else {
            self.Non_executable.deallocate(ptr, size, align);
        }
    }

    unsafe fn Get_statistics(&self) -> Statistics_type {
        Statistics_type {
            Total: self.Executable_slice.len() + self.Non_executable_slice.len(),
            Used: self.Executable.used() + self.Non_executable.used(),
            Free: self.Executable.free() + self.Non_executable.free(),
        }
    }
}

impl Protection_trait for Memory_manager_type {
    unsafe fn Set_protection(
        &self,
        address: *mut u8,
        size: usize,
        protection: Protection_type,
    ) -> bool {
        let libc_protection = Self::get_libc_protection(protection);
        let request_size = Self::round_page_size(size, self.get_page_size());

        mprotect(address as *mut c_void, request_size, libc_protection) == 0
    }
}

impl Memory_manager_type {
    const fn get_libc_protection(protection: Protection_type) -> i32 {
        let mut libc_protection = PROT_NONE;

        if protection.Get_execute() {
            libc_protection |= PROT_EXEC;
        }
        if protection.Get_read() {
            libc_protection |= PROT_READ;
        }
        if protection.Get_write() {
            libc_protection |= PROT_WRITE;
        }

        libc_protection
    }
}
