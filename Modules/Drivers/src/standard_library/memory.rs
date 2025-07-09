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
                    capabilities: Capabilities_type::New(false, false),
                    slice: &mut [],
                },
                Region_type {
                    heap: Heap::empty(),
                    capabilities: Capabilities_type::New(true, false),
                    slice: &mut [],
                },
            ])),
        }
    }
}

impl Drop for Memory_manager_type {
    fn drop(&mut self) {
        self.regions.lock(|Regions| {
            let mut regions = Regions.borrow_mut();
            for region in regions.iter_mut() {
                if !region.slice.is_empty() {
                    unsafe {
                        Unmap(region.slice.as_mut_ptr(), region.slice.len());
                    }
                }
            }
        });
    }
}

impl Manager_trait for Memory_manager_type {
    unsafe fn Allocate(
        &self,
        capabilities: Capabilities_type,
        layout: Layout_type,
    ) -> Option<NonNull<u8>> {
        self.regions.lock(|regions| {
            let mut regions = regions.try_borrow_mut().ok()?;

            // Find the first region that can satisfy the allocation request
            let Capable_regions = regions
                .iter_mut()
                .filter(|region| region.capabilities.is_superset_of(capabilities));

            // Try to allocate from the first capable region
            for Region in Capable_regions {
                let result = Region.heap.allocate_first_fit(layout).ok();
                if result.is_some() {
                    return result;
                }
            }

            let Capable_regions = regions
                .iter_mut()
                .filter(|region| region.capabilities.is_superset_of(capabilities));

            // If no region could satisfy the request, try to expand existing regions
            for Region in Capable_regions {
                // Try to expand the region to fit the requested layout
                if Expand(Region, layout.size()) {
                    // Try to allocate again after expanding
                    let Result = Region.heap.allocate_first_fit(layout).ok();
                    if Result.is_some() {
                        return Result;
                    }
                }
            }

            None
        })
    }

    unsafe fn Deallocate(&self, Pointer: NonNull<u8>, Layout: Layout_type) {
        self.regions.lock(|regions| {
            let mut regions = regions.borrow_mut();

            let Region = regions
                .iter_mut()
                .find(|region| is_slice_within_region(region, Pointer, Layout));

            if let Some(Region) = Region {
                Region.heap.deallocate(Pointer, Layout);
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

unsafe fn Expand(Region: &mut Region_type, Tried_size: usize) -> bool {
    let page_size = get_page_size();
    // If the region is empty, allocate a new one
    if Region.slice.is_empty() {
        let size = Round_page_size(Tried_size.max(INITIAL_HEAP_SIZE), page_size);
        let new_slice = Map(size, Region.capabilities);
        let new_size = new_slice.len();
        Region
            .heap
            .init(new_slice.as_mut_ptr() as *mut u8, new_size);
        Region.slice = new_slice;
        return true;
    }
    // If the region is not empty, try to expand it
    let Region_old_size = Region.slice.len();
    let new_size = Region_old_size + Tried_size;
    let new_size = Round_page_size(new_size, page_size);

    Remap(&mut Region.slice, new_size);

    let Difference = new_size - Region_old_size;

    Region.heap.extend(Difference);

    true
}

fn get_page_size() -> usize {
    unsafe { sysconf(_SC_PAGE_SIZE) as usize }
}

const fn Round_page_size(Size: usize, Page_size: usize) -> usize {
    (Size + Page_size - 1) & !(Page_size - 1) // Round up to the nearest page size
}

unsafe fn Map(size: usize, Capabilities: Capabilities_type) -> &'static mut [MaybeUninit<u8>] {
    let page_size = get_page_size();
    let size = Round_page_size(size, page_size);

    let Capabilities = if Capabilities.get_executable() {
        PROT_READ | PROT_WRITE | PROT_EXEC
    } else {
        PROT_READ | PROT_WRITE
    };

    let Pointer = mmap(
        null_mut(),
        size,
        Capabilities,
        MAP_PRIVATE | MAP_ANONYMOUS | MAP_32BIT,
        -1,
        0,
    );

    if Pointer == MAP_FAILED {
        panic!("Failed to allocate memory");
    }

    core::slice::from_raw_parts_mut(Pointer as *mut MaybeUninit<u8>, size)
}

unsafe fn Remap(Slice: &mut &'static mut [MaybeUninit<u8>], New_size: usize) {
    let page_size = get_page_size();
    let new_size = Round_page_size(New_size, page_size);

    let Old_size = Slice.len();

    let Pointer = mremap(Slice.as_mut_ptr() as *mut c_void, Old_size, new_size, 0);

    if (Pointer == MAP_FAILED) || (Pointer != Slice.as_mut_ptr() as *mut c_void) {
        panic!("Failed to reallocate memory");
    }

    *Slice = core::slice::from_raw_parts_mut(Slice.as_mut_ptr(), new_size);
}

unsafe fn Unmap(Pointer: *mut MaybeUninit<u8>, Size: usize) {
    munmap(Pointer as *mut c_void, Size);
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
        let Test_manager = Memory_manager_type::new();

        // Allocate some memory using the test manager directly
        let Layout = Layout_type::from_size_align(128, 8).unwrap();
        let Capabilities = Capabilities_type::New(false, false);

        unsafe {
            let Allocation = Test_manager.Allocate(Capabilities, Layout);
            assert!(Allocation.is_some());

            if let Some(Pointer) = Allocation {
                // Use the allocated memory (e.g., write to it)
                *Pointer.as_ptr() = 42;
                assert_eq!(*Pointer.as_ptr(), 42);

                // Deallocate the memory
                Test_manager.Deallocate(Pointer, Layout);
            }
        }
    }

    #[test]
    fn test_get_used_free() {
        unsafe {
            let Manager = Memory_manager_type::new();

            // Allocate some memory
            let Layout = Layout_type::from_size_align(128, 8).unwrap();
            let Capabilities = Capabilities_type::New(false, false);

            let Allocation = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation.is_some());

            // Check used and free memory
            let Used = Manager.get_used();
            let Free = Manager.get_free();
            assert_eq!(Used, Layout.size());
            assert_eq!(Free, INITIAL_HEAP_SIZE - Layout.size());

            // Deallocate the memory
            if let Some(Pointer) = Allocation {
                Manager.Deallocate(Pointer, Layout);
            }
        }
    }

    #[test]
    fn test_memory_manager_initialization() {
        unsafe {
            let Manager = Memory_manager_type::new();

            // Perform an initial allocation to trigger initialization
            let Layout = Layout_type::from(Layout_type::from_size_align(128, 8).unwrap());
            let Capabilities = Capabilities_type::New(false, false);
            let Allocation = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation.is_some());
            if let Some(Pointer) = Allocation {
                // Deallocate the memory
                Manager.Deallocate(Pointer, Layout);
            }

            let Capabilities = Capabilities_type::New(true, false);
            let Allocation = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation.is_some());
            if let Some(Pointer) = Allocation {
                // Deallocate the memory
                Manager.Deallocate(Pointer, Layout);
            }

            // Check that regions are initialized
            Manager.regions.lock(|Regions| {
                let Regions_reference = Regions.borrow();
                assert!(!Regions_reference[0].slice.is_empty());
                assert!(!Regions_reference[1].slice.is_empty());
                assert!(!Regions_reference[0].capabilities.get_executable());
                assert!(Regions_reference[1].capabilities.get_executable());
            });
        }
    }

    #[test]
    fn test_allocate_deallocate() {
        unsafe {
            let Manager = Memory_manager_type::new();

            // Allocate some memory
            let Layout = Layout_type::from_size_align(128, 8).unwrap();
            let Capabilities = Capabilities_type::New(false, false);

            let Allocation = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation.is_some());

            // Deallocate the memory
            if let Some(Pointer) = Allocation {
                Manager.Deallocate(Pointer, Layout);
            }
        }
    }

    #[test]
    fn test_executable_memory() {
        unsafe {
            let Manager = Memory_manager_type::new();

            // Allocate some executable memory
            let Layout = Layout_type::from_size_align(128, 8).unwrap();
            let Capabilities = Capabilities_type::New(true, false);

            let Allocation = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation.is_some());

            // Deallocate the memory
            if let Some(Pointer) = Allocation {
                Manager.Deallocate(Pointer, Layout);
            }
        }
    }

    #[test]
    fn test_memory_expansion() {
        unsafe {
            let Manager = Memory_manager_type::new();

            // Allocate a chunk of memory close to the region size
            // to trigger expansion
            let _page_size = get_page_size();
            let Layout = Layout_type::from_size_align(INITIAL_HEAP_SIZE, 8).unwrap();
            let Capabilities = Capabilities_type::New(false, false);

            let Allocation1 = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation1.is_some());

            // Try another allocation that should trigger expansion
            let Allocation2 = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation2.is_some());

            // Deallocate everything
            if let Some(Pointer) = Allocation1 {
                Manager.Deallocate(Pointer, Layout);
            }
            if let Some(Pointer) = Allocation2 {
                Manager.Deallocate(Pointer, Layout);
            }
        }
    }

    #[test]
    fn test_is_slice_within_region() {
        unsafe {
            let Manager = Memory_manager_type::new();

            // Allocate some memory
            let Layout = Layout_type::from_size_align(128, 8).unwrap();
            let Capabilities = Capabilities_type::New(false, false);

            let Allocation = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation.is_some());

            Manager.regions.lock(|Regions| {
                let Regions_reference = Regions.borrow();
                if let Some(Pointer) = Allocation {
                    // Check that the pointer is within the region
                    assert!(is_slice_within_region(
                        &Regions_reference[0],
                        Pointer,
                        Layout
                    ));

                    // Create a pointer outside the region
                    let Invalid_pointer = NonNull::new(0xdeadbeef as *mut u8).unwrap();
                    assert!(!is_slice_within_region(
                        &Regions_reference[0],
                        Invalid_pointer,
                        Layout
                    ));
                }
            });

            // Deallocate
            if let Some(Pointer) = Allocation {
                Manager.Deallocate(Pointer, Layout);
            }
        }
    }

    #[test]
    fn test_page_size_rounding() {
        let Page_size = get_page_size();

        // Test various sizes
        assert_eq!(Round_page_size(1, Page_size), Page_size);
        assert_eq!(Round_page_size(Page_size, Page_size), Page_size);
        assert_eq!(Round_page_size(Page_size + 1, Page_size), Page_size * 2);
        assert_eq!(Round_page_size(Page_size * 2, Page_size), Page_size * 2);
    }
}
