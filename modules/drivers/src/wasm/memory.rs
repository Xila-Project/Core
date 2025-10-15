//! WASM Memory Manager using Linked List Allocator
//!
//! This module provides a memory manager for WASM targets that uses the `linked_list_allocator`
//! crate to manage heap memory. The implementation uses an `Inner` struct to encapsulate the heap
//! and its state, ensuring thread safety through `CriticalSectionMutex`.
//! The heap is initialized immediately in the constructor, ensuring it's ready for use.
//!
//! # Examples
//!
//! ## Using external heap memory (initialized immediately):
//! ```rust
//! use drivers::wasm::memory::MemoryManager;
//!
//! static mut CUSTOM_HEAP: [u8; 2 * 1024 * 1024] = [0; 2 * 1024 * 1024]; // 2MB
//!
//! unsafe {
//!     let heap_ref = core::ptr::addr_of_mut!(CUSTOM_HEAP).as_mut().unwrap();
//!     let manager = MemoryManager::new(heap_ref); // Heap is ready immediately
//! }
//! ```
//!
//! ## Using the convenience function for custom size:
//! ```rust
//! use drivers::wasm::memory::create_memory_manager_with_size;
//!
//! static mut HEAP: [u8; 4096] = [0; 4096];
//!
//! unsafe {
//!     let heap_ref = core::ptr::addr_of_mut!(HEAP).as_mut().unwrap();
//!     let manager = create_memory_manager_with_size(heap_ref); // Heap is ready immediately
//! }
//! ```
//!
//! ## Using the default heap (easiest approach):
//! ```rust
//! use drivers::wasm::memory::create_default_memory_manager;
//!
//! let manager = create_default_memory_manager(); // Uses 1MB default heap, ready immediately
//! ```
use core::{cell::RefCell, ptr::NonNull};

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use linked_list_allocator::Heap;
use memory::{Capabilities, Layout, ManagerTrait, utilities::round_to_page_size};

use core::arch::wasm32::memory_grow;

unsafe extern "C" {
    static mut __heap_base: u8;
}

const fn get_heap_start() -> *mut u8 {
    &raw mut __heap_base as *mut u8
}

const PAGE_SIZE: usize = 65536;

struct Inner {
    /// The linked list allocator heap
    heap: Heap,
    /// Flag indicating if the heap has been initialized
    is_initialized: bool,
}

impl Inner {
    unsafe fn expand(&mut self, new_size: usize) -> Option<usize> {
        let new_size_rounded = round_to_page_size(new_size, PAGE_SIZE);
        let delta_page_count = new_size_rounded / PAGE_SIZE;

        if memory_grow::<0>(delta_page_count) == usize::MAX {
            return None; // failed to grow memory
        }

        let new_size = delta_page_count * PAGE_SIZE;

        Some(new_size)
    }

    unsafe fn allocate_first_fit(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        if !self.is_initialized {
            unsafe {
                let new_size = self.expand(layout.size())?;
                self.heap.init(get_heap_start(), new_size);
            }
            self.is_initialized = true;
        }

        if let Ok(pointer) = self.heap.allocate_first_fit(layout) {
            return Some(pointer);
        }

        // If allocation fails, try to expand the heap
        unsafe {
            let new_size = self.expand(layout.size())?;
            self.heap.extend(new_size);
        }

        self.heap.allocate_first_fit(layout).ok()
    }
}

pub struct MemoryManager(CriticalSectionMutex<RefCell<Inner>>);

// Safety: We ensure thread safety through CriticalSectionMutex
unsafe impl Send for MemoryManager {}
unsafe impl Sync for MemoryManager {}

impl MemoryManager {
    /// Create a new MemoryManager using the provided memory
    /// The heap is initialized immediately with the provided memory
    pub const fn new() -> Self {
        MemoryManager(CriticalSectionMutex::new(RefCell::new(Inner {
            heap: Heap::empty(),
            is_initialized: false, // This will be initialized later
        })))
    }
}

impl ManagerTrait for MemoryManager {
    unsafe fn allocate(&self, capabilities: Capabilities, layout: Layout) -> Option<NonNull<u8>> {
        if capabilities.get_direct_memory_access() {
            return None; // Direct memory access is not supported in this implementation
        }
        if capabilities.get_executable() {
            return None; // Executable memory is not supported in this implementation
        }

        self.0
            .lock(|inner| unsafe { inner.borrow_mut().allocate_first_fit(layout) })
    }

    unsafe fn deallocate(&self, pointer: std::ptr::NonNull<u8>, layout: memory::Layout) {
        self.0
            .lock(|inner| unsafe { inner.borrow_mut().heap.deallocate(pointer, layout) });
    }

    unsafe fn get_used(&self) -> usize {
        self.0.lock(|inner| {
            let inner = inner.borrow();
            inner.heap.used()
        })
    }

    unsafe fn get_free(&self) -> usize {
        self.0.lock(|inner| {
            let inner = inner.borrow();
            inner.heap.free()
        })
    }
}

#[macro_export]
macro_rules! instantiate_global_allocator {
    () => {
        static __MEMORY_MANAGER: $crate::wasm::memory::MemoryManager =
            $crate::wasm::memory::MemoryManager::new();

        $crate::memory_exported::instantiate_global_allocator!(&__MEMORY_MANAGER);
    };
}
pub use instantiate_global_allocator;
