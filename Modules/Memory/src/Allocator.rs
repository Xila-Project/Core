use core::{
    alloc::{GlobalAlloc, Layout},
    cell::{OnceCell, RefCell},
    mem::MaybeUninit,
    ptr::NonNull,
};

use critical_section::Mutex;
use linked_list_allocator::Heap;

use crate::{Capabilities_type, Region_statistics_type, Statistics_type};

/// The global allocator instance
#[global_allocator]
pub static Allocator: Allocator_type = Allocator_type::New();

#[cfg(feature = "Debug")]
struct InternalHeapStats {
    max_usage: usize,
    total_allocated: usize,
    total_freed: usize,
}

pub const Number_of_regions: usize = 5;

struct Region_type {
    pub Heap: Heap,
    pub Capabilities: Capabilities_type,
}

/// A memory allocator
///
/// In addition to what Rust's memory allocator can do it allows to allocate
/// memory in regions satisfying specific needs.
pub struct Allocator_type<'a> {
    Regions: Mutex<&'a [RefCell<Region_type>]>,
    #[cfg(feature = "Debug")]
    internal_heap_stats: Mutex<RefCell<InternalHeapStats>>,
}

impl Allocator_type {
    /// Create a new allocator
    pub const fn New() -> Self {
        Self {
            Regions: Mutex::new(RefCell::new([None, None, None, None, None])),
            #[cfg(feature = "Debug")]
            internal_heap_stats: Mutex::const_new(RefCell::new(InternalHeapStats {
                max_usage: 0,
                total_allocated: 0,
                total_freed: 0,
            })),
        }
    }

    pub fn Add_region(
        &self,
        Memory: &'static mut MaybeUninit<[u8]>,
        Capabilities: Capabilities_type,
    ) -> bool {
        critical_section::with(|Critical_section| {
            self.Regions
                .borrow_ref_mut(Critical_section)
                .iter_mut()
                .find(|Region| Region.is_none())
                .map(|Region| {
                    let Heap = Heap::new(heap_bottom, heap_size) * Region = Some(Region_type {
                        Heap: Heap::new(Heap.as_mut_ptr() as *mut u8, Heap.len()),
                        Capabilities,
                    });
                })
                .is_some()
        })
    }

    /// Returns an estimate of the amount of bytes in use in all memory regions.
    pub fn Get_used(&self) -> usize {
        critical_section::with(|Critical_section| {
            self.Regions
                .borrow_ref(Critical_section)
                .iter()
                .filter_map(|Region| Region.as_ref())
                .map(|Region| Region.Get_used())
                .sum()
        })
    }

    /// Returns an estimate of the amount of bytes available in all memory regions.
    pub fn Get_free(&self) -> usize {
        critical_section::with(|Critical_section| {
            self.Regions
                .borrow_ref(Critical_section)
                .iter()
                .filter_map(|Region| Region.as_ref())
                .map(|Region| Region.Get_free())
                .sum()
        })
    }

    /// Returns an estimate of the total amount of bytes in all memory regions.
    pub fn Get_size(&self) -> usize {
        critical_section::with(|Critical_section| {
            self.Regions
                .borrow_ref(Critical_section)
                .iter()
                .filter_map(|Region| Region.as_ref())
                .map(|Region| Region.Get_size())
                .sum()
        })
    }

    pub fn Get_statistics(&self) -> Statistics_type {
        const EMPTY_REGION_STAT: Option<Region_statistics_type> = None;
        let mut region_stats: [Option<Region_statistics_type>; 3] = [EMPTY_REGION_STAT; 3];

        critical_section::with(|Critical_section| {
            let mut used = 0;
            let mut free = 0;
            let regions = self.Regions.borrow_ref(Critical_section);
            for (id, region) in regions.iter().enumerate() {
                if let Some(region) = region.as_ref() {
                    let stats = region.stats();
                    free += stats.free;
                    used += stats.used;
                    region_stats[id] = Some(region.stats());
                }
            }

            cfg_if::cfg_if! {
                if #[cfg(feature = "Debug")] {
                    let internal_heap_stats = self.internal_heap_stats.borrow_ref(cs);
                    HeapStats {
                        region_stats,
                        size: free + used,
                        current_usage: used,
                        max_usage: internal_heap_stats.max_usage,
                        total_allocated: internal_heap_stats.total_allocated,
                        total_freed: internal_heap_stats.total_freed,
                    }
                } else {
                    HeapStats {
                        region_stats,
                        size: free + used,
                        current_usage: used,
                    }
                }
            }
        })
    }

    /// The free heap satisfying the given requirements
    pub fn Deallocate(&self, Capabilities: Capabilities_type) -> usize {
        critical_section::with(|cs| {
            let Regions = self.Regions.borrow(cs);
            let mut free = 0;
            for Region in Regions.iter().filter(|Region| {
                if Region.is_some() {
                    Region
                        .as_ref()
                        .unwrap()
                        .capabilities
                        .is_superset(Capabilities)
                } else {
                    false
                }
            }) {
                if let Some(region) = Region.as_ref() {
                    free += region.Heap.free();
                }
            }
            free
        })
    }

    pub unsafe fn Allocate(
        &self,
        capabilities: Capabilities_type,
        layout: Layout,
    ) -> Option<NonNull<u8>> {
        critical_section::with(|Critical_section| {
            #[cfg(feature = "Debug")]
            let before = self.used();

            let Result = self
                .Regions
                .borrow_ref_mut(Critical_section)
                .iter_mut()
                .find_map(|Region| {
                    if Region.is_none() {
                        return None;
                    }

                    let Region = Region.as_mut().unwrap();
                    if Region.Get_capabilities().is_superset(capabilities) {
                        Region.Allocate(layout)
                    } else {
                        None
                    }
                });

            Result.map(|Allocation| {
                #[cfg(feature = "Debug")]
                {
                    let mut internal_heap_stats =
                        self.internal_heap_stats.borrow_ref_mut(Critical_section);
                    drop(Region_type);
                    // We need to call used because [linked_list_allocator::Heap] does internal size
                    // alignment so we cannot use the size provided by the layout.
                    let used = self.used();

                    internal_heap_stats.total_allocated += used - before;
                    internal_heap_stats.max_usage =
                        core::cmp::max(internal_heap_stats.max_usage, used);
                }

                Allocation
            })
        })
    }
}

unsafe impl<'a> GlobalAlloc for Allocator_type<'a> {
    unsafe fn alloc(&self, Layout: Layout) -> *mut u8 {
        self.Allocate(Capabilities_type::New(false, false), Layout)
    }

    unsafe fn dealloc(&self, Pointer: *mut u8, Layout: Layout) {
        if Pointer.is_null() {
            return;
        }

        critical_section::with(|cs| {
            #[cfg(feature = "Debug")]
            let before = self.used();
            let mut regions = self.Regions.borrow_ref_mut(cs);
            let mut iter = (*regions).iter_mut();

            while let Some(Some(region)) = iter.next() {
                if region.Heap.bottom() <= Pointer && region.Heap.top() >= Pointer {
                    region
                        .Heap
                        .deallocate(NonNull::new_unchecked(Pointer), Layout);
                }
            }

            #[cfg(feature = "Debug")]
            {
                let mut internal_heap_stats = self.internal_heap_stats.borrow_ref_mut(cs);
                drop(regions);
                // We need to call `used()` because [linked_list_allocator::Heap] does internal
                // size alignment so we cannot use the size provided by the
                // layout.
                internal_heap_stats.total_freed += before - self.used();
            }
        })
    }
}

#[cfg(feature = "Nightly")]
unsafe impl core::alloc::Allocator for Allocator_type {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let raw_ptr = unsafe { self.alloc(layout) };

        if raw_ptr.is_null() {
            return Err(core::alloc::AllocError);
        }

        let ptr = NonNull::new(raw_ptr).ok_or(core::alloc::AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.dealloc(ptr.as_ptr(), layout);
    }
}
