use core::{
    alloc::Layout,
    cell::{Ref, RefCell},
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{null_mut, NonNull},
};

use embassy_sync::blocking_mutex::{CriticalSectionMutex, Mutex};
use libc::{
    mmap, mprotect, mremap, munmap, sysconf, MAP_32BIT, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED,
    MAP_PRIVATE, MREMAP_MAYMOVE, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE, _SC_PAGE_SIZE,
};
use linked_list_allocator::Heap;
use Memory::{
    Allocator_trait, Capabilities_type, Layout_type, Protection_trait, Protection_type,
    Statistics_type,
};

// Initial heap size and growth constants
const INITIAL_HEAP_SIZE: usize = 1024 * 1024; // 1MB
const HEAP_GROWTH_FACTOR: usize = 2;

struct Region_type {
    pub Heap: Heap,
    pub Capabilities: Capabilities_type,
    pub Slice: &'static mut [MaybeUninit<u8>],
}

pub struct Memory_manager_type {
    Regions: CriticalSectionMutex<RefCell<[Region_type; 2]>>,
}

impl Memory_manager_type {
    pub const fn New() -> Self {
        Memory_manager_type {
            Regions: Mutex::new(RefCell::new([
                Region_type {
                    Heap: Heap::empty(),
                    Capabilities: Capabilities_type::New(true, false),
                    Slice: &mut [],
                },
                Region_type {
                    Heap: Heap::empty(),
                    Capabilities: Capabilities_type::New(false, false),
                    Slice: &mut [],
                },
            ])),
        }
    }
}

impl Allocator_trait for Memory_manager_type {
    unsafe fn Allocate(
        &self,
        Capabilities: Capabilities_type,
        Layout: Layout_type,
    ) -> Option<NonNull<u8>> {
        self.Regions.lock(|Regions| {
            for Region in Regions.borrow_mut().iter_mut() {
                if Region.Capabilities.Is_superset_of(Capabilities) {
                    let Result = Region.Heap.allocate_first_fit(Layout);
                    if let Ok(Pointer) = Pointer {
                        return Some(Pointer);
                    } else {
                        // Try to expand the region if allocation fails
                        let Tried_size = Layout.size();
                        if Expand(Region, Tried_size) {
                            let Pointer = Region.Heap.allocate_first_fit(Layout);
                            if Pointer.is_some() {
                                return Some(Pointer);
                            }
                        }
                    }
                }
            }

            None
        })
    }

    unsafe fn Deallocate(&self, Pointer: NonNull<u8>, Layout: Layout_type) {
        self.Regions.lock(|Regions| {
            let mut Regions = Regions.borrow_mut();

            let Region = Regions
                .iter_mut()
                .find(|Region| Is_slice_within_region(Region, Pointer, Layout));

            if let Some(Region) = Region {
                Region.Heap.deallocate(Pointer, Layout);
            } else {
                panic!("Pointer not found in any region");
            }
        });
    }

    unsafe fn Get_used(&self) -> usize {
        self.Regions.iter().map(|region| region.Heap.used()).sum()
    }

    unsafe fn Get_free(&self) -> usize {
        self.Regions.iter().map(|region| region.Heap.free()).sum()
    }
}

unsafe fn Is_slice_within_region(
    Region: &Region_type,
    Pointer: NonNull<u8>,
    Layout: Layout_type,
) -> bool {
    let Start = Region.Slice.as_ptr() as usize;
    let End = Start + Region.Slice.len();
    let Pointer = Pointer.as_ptr() as usize;

    (Start..End).contains(&Pointer) && (Start..End).contains(&(Pointer + Layout.size()))
}

unsafe fn Expand(Region: &mut Region_type, Tried_size: usize) -> bool {
    let Page_size = Get_page_size();
    if Region.Slice.len() == 0 {
        // If the region is empty, allocate a new one
        let Size = Round_page_size(Tried_size, Page_size);
        let Slice = Region.Slice;

        Region.Slice = Map(Page_size, Region.Capabilities);
        Region.Heap.init_from_slice(Slice);
        return true;
    }

    let Region_old_size = Region.Slice.len();
    let New_size = Region_old_size + Tried_size;
    let New_size = Round_page_size(New_size, Page_size);

    let New_slice = Remap(Region.Slice, New_size);

    let Difference = New_size - Region_old_size;

    Region.Heap.extend(Difference);

    return true;
}

fn Get_page_size() -> usize {
    unsafe { sysconf(_SC_PAGE_SIZE) as usize }
}

const fn Round_page_size(Size: usize, Page_size: usize) -> usize {
    (Size + Page_size - 1) & !(Page_size - 1) // Round up to the nearest page size
}

unsafe fn Map(size: usize, Capabilities: Capabilities_type) -> &'static mut [MaybeUninit<u8>] {
    let Page_size = Get_page_size();
    let Size = Round_page_size(size, Page_size);

    let Capabilities = if Capabilities.Get_executable() {
        PROT_READ | PROT_WRITE | PROT_EXEC
    } else {
        PROT_READ | PROT_WRITE
    };

    let Pointer = mmap(
        std::ptr::null_mut(),
        Size,
        Capabilities,
        MAP_PRIVATE | MAP_ANONYMOUS | MAP_32BIT,
        -1,
        0,
    );

    if Pointer == MAP_FAILED {
        panic!("Failed to allocate memory");
    }

    std::slice::from_raw_parts_mut(Pointer as *mut MaybeUninit<u8>, Size)
}

unsafe fn Remap(
    Old_slice: &'static mut [MaybeUninit<u8>],
    New_size: usize,
) -> &'static mut [MaybeUninit<u8>] {
    let Page_size = Get_page_size();
    let New_size = Round_page_size(New_size, Page_size);

    let Old_size = Old_slice.len();

    let Pointer = mremap(
        Old_slice.as_mut_ptr() as *mut c_void,
        Old_size,
        New_size,
        0, // We don't want to move the memory
        null_mut(),
    );

    if Pointer == MAP_FAILED {
        panic!("Failed to reallocate memory");
    }

    std::slice::from_raw_parts_mut(Pointer as *mut MaybeUninit<u8>, New_size)
}

unsafe fn Unmap(Pointer: *mut MaybeUninit<u8>, Size: usize) {
    munmap(Pointer as *mut c_void, Size);
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr::NonNull;

    #[test]
    fn test_memory_manager_initialization() {
        unsafe {
            let manager = Memory_manager_type::New();

            // Check that regions are initialized
            manager.Regions.lock(|regions| {
                let regions = regions.borrow();
                assert!(regions[0].Slice.len() > 0);
                assert!(regions[1].Slice.len() > 0);
                assert!(regions[0].Capabilities.Get_executable());
                assert!(!regions[1].Capabilities.Get_executable());
            });
        }
    }

    #[test]
    fn test_allocate_deallocate() {
        unsafe {
            let mut manager = Memory_manager_type::New();

            // Allocate some memory
            let layout = Layout_type::from(Layout::from_size_align(128, 8).unwrap());
            let capabilities = Capabilities_type::New(false, false);

            let allocation = manager.Allocate(capabilities, layout);
            assert!(allocation.is_some());

            // Deallocate the memory
            if let Some(ptr) = allocation {
                manager.Deallocate(ptr, layout);
            }
        }
    }

    #[test]
    fn test_executable_memory() {
        unsafe {
            let mut manager = Memory_manager_type::New();

            // Allocate some executable memory
            let layout = Layout_type::from(Layout::from_size_align(128, 8).unwrap());
            let capabilities = Capabilities_type::New(true, false);

            let allocation = manager.Allocate(capabilities, layout);
            assert!(allocation.is_some());

            // Deallocate the memory
            if let Some(ptr) = allocation {
                manager.Deallocate(ptr, layout);
            }
        }
    }

    #[test]
    fn test_memory_expansion() {
        unsafe {
            let mut manager = Memory_manager_type::New();

            // Allocate a chunk of memory close to the region size
            // to trigger expansion
            let page_size = Get_page_size();
            let layout = Layout_type::from(Layout::from_size_align(page_size - 64, 8).unwrap());
            let capabilities = Capabilities_type::New(false, false);

            let allocation1 = manager.Allocate(capabilities, layout);
            assert!(allocation1.is_some());

            // Try another allocation that should trigger expansion
            let allocation2 = manager.Allocate(capabilities, layout);
            assert!(allocation2.is_some());

            // Deallocate everything
            if let Some(ptr) = allocation1 {
                manager.Deallocate(ptr, layout);
            }
            if let Some(ptr) = allocation2 {
                manager.Deallocate(ptr, layout);
            }
        }
    }

    #[test]
    fn test_is_slice_within_region() {
        unsafe {
            let mut manager = Memory_manager_type::New();

            // Allocate some memory
            let layout = Layout_type::from(Layout::from_size_align(128, 8).unwrap());
            let capabilities = Capabilities_type::New(false, false);

            let allocation = manager.Allocate(capabilities, layout);
            assert!(allocation.is_some());

            manager.Regions.lock(|regions| {
                let regions_ref = regions.borrow();
                if let Some(ptr) = allocation {
                    // Check that the pointer is within the region
                    assert!(Is_slice_within_region(&regions_ref[1], ptr, layout));

                    // Create a pointer outside the region
                    let invalid_ptr = NonNull::new(std::ptr::null_mut::<u8>()).unwrap();
                    assert!(!Is_slice_within_region(
                        &regions_ref[1],
                        invalid_ptr,
                        layout
                    ));
                }
            });

            // Deallocate
            if let Some(ptr) = allocation {
                manager.Deallocate(ptr, layout);
            }
        }
    }

    #[test]
    fn test_page_size_rounding() {
        let page_size = Get_page_size();

        // Test various sizes
        assert_eq!(Round_page_size(1, page_size), page_size);
        assert_eq!(Round_page_size(page_size, page_size), page_size);
        assert_eq!(Round_page_size(page_size + 1, page_size), page_size * 2);
        assert_eq!(Round_page_size(page_size * 2, page_size), page_size * 2);
    }
}
