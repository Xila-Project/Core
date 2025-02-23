use core::ptr::NonNull;
use core::{alloc::Layout, ptr::write_bytes};

use portable_atomic::{AtomicPtr, Ordering};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct Header_type(usize);

impl Header_type {
    pub fn New(Size: usize, Occupied: bool) -> Self {
        Header_type(Size | (Occupied as usize))
    }

    pub fn Get_size(&self) -> usize {
        self.0 & !1
    }

    pub fn Get_occupied(&self) -> bool {
        self.0 & 1 != 0
    }

    pub fn Set_occupied(&mut self, Occupied: bool) {
        self.0 = self.0 & !1 | (Occupied as usize);
    }

    pub fn Set_size(&mut self, Size: usize) {
        self.0 = self.0 & 1 | Size;
    }
}

struct Page_type {
    Header: Header_type,
    Next: AtomicPtr<Page_type>,
}

pub struct Linked_list_allocator_type {
    Start: *mut u8,
    Size: usize,
    First_free: AtomicPtr<Page_type>,
}

impl Linked_list_allocator_type {
    pub fn New(Start: *mut u8, Size: usize) -> Self {
        // Check alignment
        assert_eq!(Start.align_offset(align_of::<usize>()), 0);

        let Allocator = Linked_list_allocator_type {
            Start,
            Size,
            First_free: AtomicPtr::new(Start as *mut Page_type),
        };

        unsafe {
            let First_free = Allocator.First_free.load(Ordering::Relaxed);
            (*First_free).Header = Header_type::New(Size, false);
            (*First_free).Next.store(First_free, Ordering::Relaxed);
        }

        Allocator
    }

    pub unsafe fn Allocate(&self, Layout: Layout) -> Option<NonNull<u8>> {
        let Layout = Layout.align_to(align_of::<usize>()).unwrap().pad_to_align();
        let Size = Layout.size();

        let mut Current_page = self.First_free.load(Ordering::Relaxed);

        loop {
            let Current_header = unsafe { &mut (*Current_page).Header };

            // If the current page is not occupied and has enough space
            if !Current_header.Get_occupied() && Current_header.Get_size() >= Size {
                let Remaining_size = Current_header.Get_size() - Size;

                if Remaining_size > 0 {
                    let Remaining = unsafe { Current_page.add(Size) };
                    (*Remaining).Header = Header_type::New(Remaining_size, false);
                    (*Remaining).Next =
                        AtomicPtr::new((*Current_page).Next.load(Ordering::Relaxed));

                    (*Current_page).Header.Set_size(Size);
                    (*Current_page).Next.store(Remaining, Ordering::Relaxed);
                }

                Current_header.Set_occupied(true);

                return Some(NonNull::new(Current_page as *mut u8).unwrap());
            }

            if Current_page == self.First_free.load(Ordering::Relaxed) {
                return None;
            }

            Current_page = unsafe { (*Current_page).Next.load(Ordering::Relaxed) };
        }
    }

    pub unsafe fn Deallocate(&self, Address: NonNull<u8>, Layout: Layout) -> bool {
        let mut Current = self.First_free.load(Ordering::Relaxed);

        let mut Previous = Current;

        loop {
            let Current_header = unsafe { &mut (*Current).Header };

            if Current as *mut u8 == Address.as_ptr() {
                Current_header.Set_occupied(false);

                let Next = (*Current).Next.load(Ordering::Relaxed);

                if !(*Next).Header.Get_occupied() {
                    Current_header.Set_size(Current_header.Get_size() + (*Next).Header.Get_size());
                    (*Current)
                        .Next
                        .store((*Next).Next.load(Ordering::Relaxed), Ordering::Relaxed);
                }

                if !(*Previous).Header.Get_occupied() {
                    (*Previous)
                        .Header
                        .Set_size((*Previous).Header.Get_size() + Current_header.Get_size());
                    (*Previous)
                        .Next
                        .store((*Current).Next.load(Ordering::Relaxed), Ordering::Relaxed);
                }

                return true;
            }

            if Current == self.First_free.load(Ordering::Relaxed) {
                return false;
            }

            Previous = Current;
            Current = unsafe { (*Current).Next.load(Ordering::Relaxed) };
        }
    }
}
