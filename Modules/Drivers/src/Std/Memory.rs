use core::{
    alloc::Layout,
    cell::{Ref, RefCell},
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{null_mut, NonNull},
};

use libc::{
    mmap, mprotect, mremap, munmap, sysconf, MAP_32BIT, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED,
    MAP_PRIVATE, MREMAP_MAYMOVE, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE, _SC_PAGE_SIZE,
};
use linked_list_allocator::Heap;
use Memory::{
    Capabilities_type, Layout_type, Manager_trait, Protection_trait, Protection_type,
    Statistics_type,
};
use Synchronization::blocking_mutex::{CriticalSectionMutex, Mutex};

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
                    Capabilities: Capabilities_type::New(false, false),
                    Slice: &mut [],
                },
                Region_type {
                    Heap: Heap::empty(),
                    Capabilities: Capabilities_type::New(true, false),
                    Slice: &mut [],
                },
            ])),
        }
    }
}

impl Manager_trait for Memory_manager_type {
    unsafe fn Allocate(
        &self,
        Capabilities: Capabilities_type,
        Layout: Layout_type,
    ) -> Option<NonNull<u8>> {
        self.Regions.lock(|Regions| {
            for Region in Regions.borrow_mut().iter_mut() {
                if Region.Capabilities.Is_superset_of(Capabilities) {
                    let Result = Region.Heap.allocate_first_fit(Layout);
                    if let Ok(Pointer) = Result {
                        return Some(Pointer);
                    } else {
                        // Try to expand the region if allocation fails
                        let Tried_size = Layout.size();
                        if Expand(Region, Tried_size) {
                            let Pointer = Region.Heap.allocate_first_fit(Layout);
                            if let Ok(Pointer) = Pointer {
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
        self.Regions.lock(|Regions| {
            let Regions = Regions.borrow();
            let mut Used = 0;
            for Region in Regions.iter() {
                Used += Region.Heap.used();
            }
            Used
        })
    }

    unsafe fn Get_free(&self) -> usize {
        self.Regions.lock(|Regions| {
            let Regions = Regions.borrow();
            let mut Free = 0;
            for Region in Regions.iter() {
                Free += Region.Heap.free();
            }
            Free
        })
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
    // If the region is empty, allocate a new one
    if Region.Slice.len() == 0 {
        let Size = Round_page_size(Tried_size.max(INITIAL_HEAP_SIZE), Page_size);
        let New_slice = Map(Size, Region.Capabilities);
        let New_size = New_slice.len();
        Region
            .Heap
            .init(New_slice.as_mut_ptr() as *mut u8, New_size);
        Region.Slice = New_slice;
        return true;
    }
    // If the region is not empty, try to expand it
    let Region_old_size = Region.Slice.len();
    let New_size = Region_old_size + Tried_size;
    let New_size = Round_page_size(New_size, Page_size);

    let New_slice = Remap(&mut Region.Slice, New_size);

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

unsafe fn Remap(Slice: &mut &'static mut [MaybeUninit<u8>], New_size: usize) {
    let Page_size = Get_page_size();
    let New_size = Round_page_size(New_size, Page_size);

    let Old_size = Slice.len();

    let Pointer = mremap(
        Slice.as_mut_ptr() as *mut c_void,
        Old_size,
        New_size,
        0, // We don't want to move the memory
        null_mut::<u8>(),
    );

    if Pointer == MAP_FAILED {
        panic!("Failed to reallocate memory");
    }

    *Slice = std::slice::from_raw_parts_mut(Pointer as *mut MaybeUninit<u8>, New_size);
}

unsafe fn Unmap(Pointer: *mut MaybeUninit<u8>, Size: usize) {
    munmap(Pointer as *mut c_void, Size);
}

#[cfg(test)]
mod Tests {
    use super::*;
    extern crate alloc;
    use alloc::alloc::{alloc, dealloc};
    use core::ptr::NonNull;
    use Memory::Allocator_type;

    #[test]
    fn Test_global_allocator() {
        #[global_allocator]
        static ALLOCATOR: Allocator_type<Memory_manager_type> =
            Allocator_type::New(Memory_manager_type::New());

        // Allocate some memory using the global allocator
        let Layout = Layout::from_size_align(128, 8).unwrap();
        let Pointer = unsafe { alloc(Layout) };

        assert!(!Pointer.is_null());
        // Use the allocated memory (e.g., write to it)
        unsafe {
            *(Pointer as *mut u8) = 42;
            assert_eq!(*(Pointer as *mut u8), 42);
        }

        // Deallocate the memory
        unsafe {
            dealloc(Pointer, Layout);
        }
    }

    #[test]
    fn Test_memory_manager_initialization() {
        unsafe {
            let Manager = Memory_manager_type::New();

            // Perform an initial allocation to trigger initialization
            let Layout = Layout_type::from(Layout::from_size_align(128, 8).unwrap());
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
            Manager.Regions.lock(|Regions| {
                let Regions_reference = Regions.borrow();
                assert!(Regions_reference[0].Slice.len() > 0);
                assert!(Regions_reference[1].Slice.len() > 0);
                assert!(!Regions_reference[0].Capabilities.Get_executable());
                assert!(Regions_reference[1].Capabilities.Get_executable());
            });
        }
    }

    #[test]
    fn Test_allocate_deallocate() {
        unsafe {
            let mut Manager = Memory_manager_type::New();

            // Allocate some memory
            let Layout = Layout_type::from(Layout::from_size_align(128, 8).unwrap());
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
    fn Test_executable_memory() {
        unsafe {
            let mut Manager = Memory_manager_type::New();

            // Allocate some executable memory
            let Layout = Layout_type::from(Layout::from_size_align(128, 8).unwrap());
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
    fn Test_memory_expansion() {
        unsafe {
            let mut Manager = Memory_manager_type::New();

            // Allocate a chunk of memory close to the region size
            // to trigger expansion
            let Page_size = Get_page_size();
            let Layout = Layout_type::from(Layout::from_size_align(Page_size - 64, 8).unwrap());
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
    fn Test_is_slice_within_region() {
        unsafe {
            let mut Manager = Memory_manager_type::New();

            // Allocate some memory
            let Layout = Layout_type::from(Layout::from_size_align(128, 8).unwrap());
            let Capabilities = Capabilities_type::New(false, false);

            let Allocation = Manager.Allocate(Capabilities, Layout);
            assert!(Allocation.is_some());

            Manager.Regions.lock(|Regions| {
                let Regions_reference = Regions.borrow();
                if let Some(Pointer) = Allocation {
                    // Check that the pointer is within the region
                    assert!(Is_slice_within_region(
                        &Regions_reference[0],
                        Pointer,
                        Layout
                    ));

                    // Create a pointer outside the region
                    let Invalid_pointer = NonNull::new(0xdeadbeef as *mut u8).unwrap();
                    assert!(!Is_slice_within_region(
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
    fn Test_page_size_rounding() {
        let Page_size = Get_page_size();

        // Test various sizes
        assert_eq!(Round_page_size(1, Page_size), Page_size);
        assert_eq!(Round_page_size(Page_size, Page_size), Page_size);
        assert_eq!(Round_page_size(Page_size + 1, Page_size), Page_size * 2);
        assert_eq!(Round_page_size(Page_size * 2, Page_size), Page_size * 2);
    }
}
