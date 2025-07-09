use core::{
    cell::RefCell,
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{null_mut, NonNull},
};

use libc::{
    mmap, mremap, munmap, sysconf, MAP_32BIT, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC,
    PROT_READ, PROT_WRITE, _SC_PAGE_SIZE,
};
use linked_list_allocator::Heap;
use memory::{Capabilities_type, Layout_type, Manager_trait};
use synchronization::blocking_mutex::{CriticalSectionMutex, Mutex};

// Initial heap size and growth constants
const INITIAL_HEAP_SIZE: usize = 1024 * 1024; // 1MB

struct Region_type {
    pub heap: Heap,
    pub capabilities: Capabilities_type,
    pub slice: &'static mut [MaybeUninit<u8>],
}

pub struct Memory_manager_type {
    regions: CriticalSectionMutex<RefCell<[Region_type; 2]>>,
}

impl Memory_manager_type {
    pub const fn new() -> Self {
        Memory_manager_type {
            regions: Mutex::new(RefCell::new([
                Region_type {
                    heap: Heap::empty(),
                    capabilities: Capabilities_type::new(false, false),
                    slice: &mut [],
                },
                Region_type {
                    heap: Heap::empty(),
                    capabilities: Capabilities_type::new(true, false),
                    slice: &mut [],
                },
            ])),
        }
    }
}

impl Drop for Memory_manager_type {
    fn drop(&mut self) {
        self.regions.lock(|regions| {
            let mut regions = regions.borrow_mut();
            for region in regions.iter_mut() {
                if !region.slice.is_empty() {
                    unsafe {
                        unmap(region.slice.as_mut_ptr(), region.slice.len());
                    }
                }
            }
        });
    }
}

impl Manager_trait for Memory_manager_type {
    unsafe fn allocate(
        &self,
        capabilities: Capabilities_type,
        layout: Layout_type,
    ) -> Option<NonNull<u8>> {
        self.regions.lock(|regions| {
            let mut regions = regions.try_borrow_mut().ok()?;

            // Find the first region that can satisfy the allocation request
            let capable_regions = regions
                .iter_mut()
                .filter(|region| region.capabilities.is_superset_of(capabilities));

            // Try to allocate from the first capable region
            for region in capable_regions {
                let result = region.heap.allocate_first_fit(layout).ok();
                if result.is_some() {
                    return result;
                }
            }

            let capable_regions = regions
                .iter_mut()
                .filter(|region| region.capabilities.is_superset_of(capabilities));

            // If no region could satisfy the request, try to expand existing regions
            for region in capable_regions {
                // Try to expand the region to fit the requested layout
                if expand(region, layout.size()) {
                    // Try to allocate again after expanding
                    let result = region.heap.allocate_first_fit(layout).ok();
                    if result.is_some() {
                        return result;
                    }
                }
            }

            None
        })
    }

    unsafe fn deallocate(&self, pointer: NonNull<u8>, layout: Layout_type) {
        self.regions.lock(|regions| {
            let mut regions = regions.borrow_mut();

            let region = regions
                .iter_mut()
                .find(|region| is_slice_within_region(region, pointer, layout));

            if let Some(region) = region {
                region.heap.deallocate(pointer, layout);
            } else {
                panic!("Pointer not found in any region");
            }
        });
    }

    unsafe fn get_used(&self) -> usize {
        self.regions.lock(|regions| {
            let regions = regions.borrow();
            let mut used = 0;
            for region in regions.iter() {
                used += region.heap.used();
            }
            used
        })
    }

    unsafe fn get_free(&self) -> usize {
        self.regions.lock(|regions| {
            let regions = regions.borrow();
            let mut free = 0;
            for region in regions.iter() {
                free += region.heap.free();
            }
            free
        })
    }

    fn get_page_size(&self) -> usize {
        get_page_size()
    }
}

unsafe fn is_slice_within_region(
    region: &Region_type,
    pointer: NonNull<u8>,
    layout: Layout_type,
) -> bool {
    let start = region.slice.as_ptr() as usize;
    let end = start + region.slice.len();
    let pointer_start = pointer.as_ptr() as usize;
    let pointer_end = pointer_start + layout.size();

    // Check if the pointer range is completely contained within the region
    pointer_start >= start && pointer_end <= end
}

unsafe fn expand(region: &mut Region_type, tried_size: usize) -> bool {
    let page_size = get_page_size();
    // If the region is empty, allocate a new one
    if region.slice.is_empty() {
        let size = round_page_size(tried_size.max(INITIAL_HEAP_SIZE), page_size);
        let new_slice = map(size, region.capabilities);
        let new_size = new_slice.len();
        region
            .heap
            .init(new_slice.as_mut_ptr() as *mut u8, new_size);
        region.slice = new_slice;
        return true;
    }
    // If the region is not empty, try to expand it
    let region_old_size = region.slice.len();
    let new_size = region_old_size + tried_size;
    let new_size = round_page_size(new_size, page_size);

    remap(&mut region.slice, new_size);

    let difference = new_size - region_old_size;

    region.heap.extend(difference);

    true
}

fn get_page_size() -> usize {
    unsafe { sysconf(_SC_PAGE_SIZE) as usize }
}

const fn round_page_size(size: usize, page_size: usize) -> usize {
    (size + page_size - 1) & !(page_size - 1) // Round up to the nearest page size
}

unsafe fn map(size: usize, capabilities: Capabilities_type) -> &'static mut [MaybeUninit<u8>] {
    let page_size = get_page_size();
    let size = round_page_size(size, page_size);

    let capabilities = if capabilities.get_executable() {
        PROT_READ | PROT_WRITE | PROT_EXEC
    } else {
        PROT_READ | PROT_WRITE
    };

    let pointer = mmap(
        null_mut(),
        size,
        capabilities,
        MAP_PRIVATE | MAP_ANONYMOUS | MAP_32BIT,
        -1,
        0,
    );

    if pointer == MAP_FAILED {
        panic!("Failed to allocate memory");
    }

    core::slice::from_raw_parts_mut(pointer as *mut MaybeUninit<u8>, size)
}

unsafe fn remap(slice: &mut &'static mut [MaybeUninit<u8>], new_size: usize) {
    let page_size = get_page_size();
    let new_size = round_page_size(new_size, page_size);

    let old_size = slice.len();

    let pointer = mremap(slice.as_mut_ptr() as *mut c_void, old_size, new_size, 0);

    if (pointer == MAP_FAILED) || (pointer != slice.as_mut_ptr() as *mut c_void) {
        panic!("Failed to reallocate memory");
    }

    *slice = core::slice::from_raw_parts_mut(slice.as_mut_ptr(), new_size);
}

unsafe fn unmap(pointer: *mut MaybeUninit<u8>, size: usize) {
    munmap(pointer as *mut c_void, size);
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::ptr::NonNull;
    use memory::Instantiate_global_allocator;

    Instantiate_global_allocator!(Memory_manager_type);

    #[test]
    fn test_global_allocator() {
        // Create a separate instance for testing to avoid reentrancy issues with the global allocator
        let test_manager = Memory_manager_type::new();

        // Allocate some memory using the test manager directly
        let layout = Layout_type::from_size_align(128, 8).unwrap();
        let capabilities = Capabilities_type::new(false, false);

        unsafe {
            let allocation = test_manager.allocate(capabilities, layout);
            assert!(allocation.is_some());

            if let Some(pointer) = allocation {
                // Use the allocated memory (e.g., write to it)
                *pointer.as_ptr() = 42;
                assert_eq!(*pointer.as_ptr(), 42);

                // Deallocate the memory
                test_manager.deallocate(pointer, layout);
            }
        }
    }

    #[test]
    fn test_get_used_free() {
        unsafe {
            let manager = Memory_manager_type::new();

            // Allocate some memory
            let layout = Layout_type::from_size_align(128, 8).unwrap();
            let capabilities = Capabilities_type::new(false, false);

            let allocation = manager.allocate(capabilities, layout);
            assert!(allocation.is_some());

            // Check used and free memory
            let used = manager.get_used();
            let free = manager.get_free();
            assert_eq!(used, layout.size());
            assert_eq!(free, INITIAL_HEAP_SIZE - layout.size());

            // Deallocate the memory
            if let Some(pointer) = allocation {
                manager.deallocate(pointer, layout);
            }
        }
    }

    #[test]
    fn test_memory_manager_initialization() {
        unsafe {
            let manager = Memory_manager_type::new();

            // Perform an initial allocation to trigger initialization
            let layout = Layout_type::from(Layout_type::from_size_align(128, 8).unwrap());
            let capabilities = Capabilities_type::new(false, false);
            let allocation = manager.allocate(capabilities, layout);
            assert!(allocation.is_some());
            if let Some(pointer) = allocation {
                // Deallocate the memory
                manager.deallocate(pointer, layout);
            }

            let capabilities = Capabilities_type::new(true, false);
            let allocation = manager.allocate(capabilities, layout);
            assert!(allocation.is_some());
            if let Some(pointer) = allocation {
                // Deallocate the memory
                manager.deallocate(pointer, layout);
            }

            // Check that regions are initialized
            manager.regions.lock(|regions| {
                let regions_reference = regions.borrow();
                assert!(!regions_reference[0].slice.is_empty());
                assert!(!regions_reference[1].slice.is_empty());
                assert!(!regions_reference[0].capabilities.get_executable());
                assert!(regions_reference[1].capabilities.get_executable());
            });
        }
    }

    #[test]
    fn test_allocate_deallocate() {
        unsafe {
            let manager = Memory_manager_type::new();

            // Allocate some memory
            let layout = Layout_type::from_size_align(128, 8).unwrap();
            let capabilities = Capabilities_type::new(false, false);

            let allocation = manager.allocate(capabilities, layout);
            assert!(allocation.is_some());

            // Deallocate the memory
            if let Some(pointer) = allocation {
                manager.deallocate(pointer, layout);
            }
        }
    }

    #[test]
    fn test_executable_memory() {
        unsafe {
            let manager = Memory_manager_type::new();

            // Allocate some executable memory
            let layout = Layout_type::from_size_align(128, 8).unwrap();
            let capabilities = Capabilities_type::new(true, false);

            let allocation = manager.allocate(capabilities, layout);
            assert!(allocation.is_some());

            // Deallocate the memory
            if let Some(pointer) = allocation {
                manager.deallocate(pointer, layout);
            }
        }
    }

    #[test]
    fn test_memory_expansion() {
        unsafe {
            let manager = Memory_manager_type::new();

            // Allocate a chunk of memory close to the region size
            // to trigger expansion
            let _page_size = get_page_size();
            let layout = Layout_type::from_size_align(INITIAL_HEAP_SIZE, 8).unwrap();
            let capabilities: Capabilities_type = Capabilities_type::new(false, false);

            let allocation1 = manager.allocate(capabilities, layout);
            assert!(allocation1.is_some());

            // Try another allocation that should trigger expansion
            let allocation2 = manager.allocate(capabilities, layout);
            assert!(allocation2.is_some());

            // Deallocate everything
            if let Some(pointer) = allocation1 {
                manager.deallocate(pointer, layout);
            }
            if let Some(pointer) = allocation2 {
                manager.deallocate(pointer, layout);
            }
        }
    }

    #[test]
    fn test_is_slice_within_region() {
        unsafe {
            let manager = Memory_manager_type::new();

            // Allocate some memory
            let layout = Layout_type::from_size_align(128, 8).unwrap();
            let capabilities = Capabilities_type::new(false, false);

            let allocation = manager.allocate(capabilities, layout);
            assert!(allocation.is_some());

            manager.regions.lock(|regions| {
                let regions_reference = regions.borrow();
                if let Some(pointer) = allocation {
                    // Check that the pointer is within the region
                    assert!(is_slice_within_region(
                        &regions_reference[0],
                        pointer,
                        layout
                    ));

                    // Create a pointer outside the region
                    let invalid_pointer = NonNull::new(0xdeadbeef as *mut u8).unwrap();
                    assert!(!is_slice_within_region(
                        &regions_reference[0],
                        invalid_pointer,
                        layout
                    ));
                }
            });

            // Deallocate
            if let Some(pointer) = allocation {
                manager.deallocate(pointer, layout);
            }
        }
    }

    #[test]
    fn test_page_size_rounding() {
        let page_size = get_page_size();

        // Test various sizes
        assert_eq!(round_page_size(1, page_size), page_size);
        assert_eq!(round_page_size(page_size, page_size), page_size);
        assert_eq!(round_page_size(page_size + 1, page_size), page_size * 2);
        assert_eq!(round_page_size(page_size * 2, page_size), page_size * 2);
    }
}
