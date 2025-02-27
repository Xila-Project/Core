#[cfg(feature = "nightly")]
use core::alloc::{AllocError, Allocator};
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::RefCell,
    fmt::Display,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use critical_section::Mutex;
use linked_list_allocator::Heap;

use crate::{Capabilities_type, Region_statistics_type};

/// The global allocator instance
#[global_allocator]
pub static HEAP: Allocator_type = Allocator_type::empty();

const NON_REGION: Option<Region_type> = None;

pub struct Region_type {
    Heap: RefCell<Heap>,
    Capabilities: Capabilities_type,
}

impl Region_type {
    pub unsafe fn New(
        Memory: &'static mut [MaybeUninit<u8>],
        Capabilities: Capabilities_type,
    ) -> Self {
        let mut Heap = Heap::empty();
        Heap.init_from_slice(Memory);

        Self { Heap, Capabilities }
    }

    pub const fn Get_capabilities(&self) -> Capabilities_type {
        self.Capabilities
    }

    /// Return stats for the current memory region
    pub fn Get_statistics(&self) -> Region_statistics_type {
        Region_statistics_type {
            Size: self.Heap.size(),
            Used: self.Heap.used(),
            Free: self.Heap.free(),
        }
    }
}

/// Stats for a heap allocator
///
/// Enable the "Debug" feature if you want collect additional heap
/// informations at the cost of extra cpu time during every alloc/dealloc.
#[derive(Debug)]
pub struct HeapStats {
    /// Granular stats for all the configured memory regions.
    region_stats: [Option<Region_statistics_type>; 3],

    /// Total size of all combined heap regions in bytes.
    size: usize,

    /// Current usage of the heap across all configured regions in bytes.
    current_usage: usize,

    /// Estimation of the max used heap in bytes.
    #[cfg(feature = "Debug")]
    max_usage: usize,

    /// Estimation of the total allocated bytes since initialization.
    #[cfg(feature = "Debug")]
    total_allocated: usize,

    /// Estimation of the total freed bytes since initialization.
    #[cfg(feature = "Debug")]
    total_freed: usize,
}

impl Display for HeapStats {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "HEAP INFO")?;
        writeln!(f, "Size: {}", self.size)?;
        writeln!(f, "Current usage: {}", self.current_usage)?;
        #[cfg(feature = "Debug")]
        {
            writeln!(f, "Max usage: {}", self.max_usage)?;
            writeln!(f, "Total freed: {}", self.total_freed)?;
            writeln!(f, "Total allocated: {}", self.total_allocated)?;
        }
        writeln!(f, "Memory Layout: ")?;
        for region in self.region_stats.iter() {
            if let Some(region) = region.as_ref() {
                region.fmt(f)?;
                writeln!(f)?;
            } else {
                // Display unused memory regions
                write!(f, "Unused   | ")?;
                write_bar(f, 0)?;
                writeln!(f, " |")?;
            }
        }
        Ok(())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HeapStats {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(fmt, "HEAP INFO\n");
        defmt::write!(fmt, "Size: {}\n", self.size);
        defmt::write!(fmt, "Current usage: {}\n", self.current_usage);
        #[cfg(feature = "Debug")]
        {
            defmt::write!(fmt, "Max usage: {}\n", self.max_usage);
            defmt::write!(fmt, "Total freed: {}\n", self.total_freed);
            defmt::write!(fmt, "Total allocated: {}\n", self.total_allocated);
        }
        defmt::write!(fmt, "Memory Layout:\n");
        for region in self.region_stats.iter() {
            if let Some(region) = region.as_ref() {
                defmt::write!(fmt, "{}\n", region);
            } else {
                defmt::write!(fmt, "Unused   | ");
                write_bar_defmt(fmt, 0);
                defmt::write!(fmt, " |\n");
            }
        }
    }
}

/// Internal stats to keep track across multiple regions.
#[cfg(feature = "Debug")]
struct InternalHeapStats {
    max_usage: usize,
    total_allocated: usize,
    total_freed: usize,
}

/// A memory allocator
///
/// In addition to what Rust's memory allocator can do it allows to allocate
/// memory in regions satisfying specific needs.
pub struct Allocator_type<'a> {
    Regions: Mutex<&'a [Region_type]>,
    #[cfg(feature = "Debug")]
    internal_heap_stats: Mutex<RefCell<InternalHeapStats>>,
}

impl<'a> Allocator_type<'a> {
    /// Create a new allocator
    pub const fn New(Regions: &'a [Region_type]) -> Self {
        Self {
            Regions: Mutex::new(Regions),
            #[cfg(feature = "Debug")]
            internal_heap_stats: Mutex::const_new(RefCell::new(InternalHeapStats {
                max_usage: 0,
                total_allocated: 0,
                total_freed: 0,
            })),
        }
    }

    /// Returns an estimate of the amount of bytes in use in all memory regions.
    pub fn used(&self) -> usize {
        critical_section::with(|cs| {
            let regions = self.Regions.borrow_ref(cs);
            let mut used = 0;
            for region in regions.iter() {
                if let Some(region) = region.as_ref() {
                    used += region.Heap.used();
                }
            }
            used
        })
    }

    pub fn stats(&self) -> HeapStats {
        const EMPTY_REGION_STAT: Option<Region_statistics_type> = None;
        let mut region_stats: [Option<Region_statistics_type>; 3] = [EMPTY_REGION_STAT; 3];

        critical_section::with(|cs| {
            let mut used = 0;
            let mut free = 0;
            let regions = self.Regions.borrow_ref(cs);
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
            let mut Regions = self.Regions.borrow(Critical_section);
            let mut Iterate = (*Regions)
                .iter_mut()
                .filter(|Region| Region.Capabilities.Is_superset_of(capabilities));

            let Result = loop {
                if let Some(region) = Iterate.next() {
                    let res = region.Heap.allocate_first_fit(layout);
                    if let Ok(res) = res {
                        break Some(res);
                    }
                } else {
                    break None;
                }
            };

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

#[cfg(feature = "nightly")]
unsafe impl Allocator for Allocator_type {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let raw_ptr = unsafe { self.alloc(layout) };

        if raw_ptr.is_null() {
            return Err(AllocError);
        }

        let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.dealloc(ptr.as_ptr(), layout);
    }
}
