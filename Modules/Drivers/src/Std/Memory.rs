use core::{
    alloc::Layout,
    cell::{Ref, RefCell},
    mem::MaybeUninit,
    os::raw::c_void,
    ptr::NonNull,
};
use std::collections::btree_set::Difference;

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

use crate::{Flags_type, Memory_allocator_trait, Protection_type};

// Initial heap size and growth constants
const INITIAL_HEAP_SIZE: usize = 1024 * 1024; // 1MB
const HEAP_GROWTH_FACTOR: usize = 2;

struct Region_type {
    pub Heap: Heap,
    pub Capabilities: Capabilities_type,
    pub Slice: &'static mut [MaybeUninit<u8>],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    unsafe fn Initialize(&self) {
        let Regions = self.Regions.lock(|Regions| {
            let Regions = Regions.borrow_mut();

            // Initialize executable heap
            Regions[0].Slice = Map(
                INITIAL_HEAP_SIZE,
                Protection_type::new_with_execute(true, true, true),
            );
            Regions[0].Heap.init_from_slice(Regions[0].Slice);

            // Initialize non-executable heap
            Regions[1].Slice = Map(
                INITIAL_HEAP_SIZE,
                Protection_type::new_with_execute(false, true, true),
            );
            Regions[1].Heap.init_from_slice(Regions[1].Slice);
        });
    }

    unsafe fn Expand(Region: &mut Region_type, Tried_size: usize) -> bool {
        let New_size = Tried_size - Region.Heap.free() + Region.Heap.size();

        let New_slice = Remap(Region.Slice, New_size);

        let Difference = New_size - Region.Slice.len();

        Region.Heap.extend(Difference);
    }

    unsafe fn Get_region<N>(
        Regions: &mut [Region_type; N],
        Pointer: NonNull<u8>,
        Layout: Layout_type,
    ) -> Option<&mut Region_type> {
        Regions.iter_mut().find(|Region| {
            let Start = Region.Slice.as_ptr() as usize;
            let End = Start + Region.Slice.len();

            (Start..End).contains(&(Pointer.as_ptr() as usize))
        })
    }
}

impl Allocator_trait for Memory_manager_type {
    unsafe fn Allocate(
        &mut self,
        Capabilities: Capabilities_type,
        Layout: Layout_type,
    ) -> Option<NonNull<u8>> {
        let mut Regions = self.Regions.lock(|Regions| {
            let Regions = Regions.borrow_mut();

            let mut Region = Regions.iter_mut().find(|Region| {
                Capabilities.Is_subset_of(Region.Capabilities)
                    && Region.Heap.allocate_first_fit(Layout).is_some()
            });

            if let Some(Region) = Region {
                return Region;
            }

            let mut Region = Regions.iter_mut().find(|Region| {
                Capabilities.Is_subset_of(Region.Capabilities)
                    && Self::Expand(Region, size)
                    && Region.Heap.allocate_first_fit(Layout).is_some()
            });

            if let Some(Region) = Region {
                return Region;
            }

            None
        });

        if let Some(Region) = Regions {
            let ptr = Region.Heap.allocate_first_fit(size, align).unwrap();
            Some(NonNull::new_unchecked(ptr as *mut u8))
        } else {
            None
        }
    }

    unsafe fn Deallocate(&mut self, Pointer: NonNull<u8>, Layout: Layout_type) {
        let Regions = self.Regions.lock(|Regions| {
            let Regions = Regions.borrow_mut();

            let Region = Self::Get_region(&mut *Regions, Pointer, Layout).expect("Invalid pointer");

            Region.Heap.deallocate(Pointer.as_ptr(), Layout.size());
        });
    }

    unsafe fn Get_used(&self) -> usize {
        self.Regions.iter().map(|region| region.Heap.used()).sum()
    }

    unsafe fn Get_free(&self) -> usize {
        self.Regions.iter().map(|region| region.Heap.free()).sum()
    }
}

impl Memory_manager_type {
    const fn To_libc_protection(protection: Protection_type) -> i32 {
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

fn Get_page_size() -> usize {
    unsafe { sysconf(_SC_PAGE_SIZE) as usize }
}

const fn Round_page_size(Size: usize, Page_size: usize) -> usize {
    (Size + Page_size - 1) & !(Page_size - 1) // Round up to the nearest page size
}

unsafe fn Map(size: usize, protection: Protection_type) -> &'static mut [MaybeUninit<u8>] {
    let page_size = Self::Get_page_size();
    let size = Self::Round_page_size(size, page_size);

    let pOINTER = mmap(
        std::ptr::null_mut(),
        size,
        Self::To_libc_protection(protection),
        MAP_PRIVATE | MAP_ANONYMOUS | MAP_32BIT,
        -1,
        0,
    );

    if pOINTER == MAP_FAILED {
        panic!("Failed to allocate memory");
    }

    std::slice::from_raw_parts_mut(pOINTER as *mut u8, size)
}

unsafe fn Remap(
    Old_slice: &'static mut [MaybeUninit<u8>],
    New_size: usize,
) -> &'static mut [MaybeUninit<u8>] {
    let Page_size = Self::Get_page_size();
    let New_size = Self::Round_page_size(New_size, Page_size);

    let Old_size = Old_slice.len();

    let Pointer = mremap(
        Old_slice.as_mut_ptr() as *mut c_void,
        Old_size,
        New_size,
        0, // We don't want to move the memory
        std::ptr::null_mut(),
    );

    if Pointer == MAP_FAILED {
        panic!("Failed to reallocate memory");
    }

    std::slice::from_raw_parts_mut(Pointer as *mut MaybeUninit<u8>, New_size)
}

unsafe fn Unmap(Pointer: *mut MaybeUninit<u8>, Size: usize) {
    munmap(Pointer as *mut c_void, Size);
}
