use std::sync::{Mutex, MutexGuard};

#[derive(Debug)]
struct Metadata_type<'a> {
    #[allow(dead_code)]
    pub Guard: MutexGuard<'a, ()>,
    pub Thread: std::thread::ThreadId,
}

#[derive(Debug)]
pub struct Raw_mutex_type<'a> {
    Mutex: Mutex<()>,
    Metadata: Option<Metadata_type<'a>>,
    Recursive: bool,
}

impl<'a> Raw_mutex_type<'a> {
    pub fn New(Recursive: bool) -> Self {
        println!("Creating Raw_mutex");

        Self {
            Mutex: Mutex::new(()),
            Metadata: None,
            Recursive,
        }
    }

    pub fn Is_valid_pointer(Pointer: *const Raw_mutex_type<'a>) -> bool {
        if Pointer.is_null() {
            return false;
        }

        if Pointer as usize % std::mem::align_of::<Self>() != 0 {
            return false;
        }

        true
    }

    pub unsafe fn From_pointer(Pointer: *const Raw_mutex_type<'a>) -> Option<&'a Self> {
        if Self::Is_valid_pointer(Pointer) {
            return None;
        }

        Some(&*Pointer)
    }

    pub unsafe fn From_mutable_pointer(Pointer: *mut Raw_mutex_type<'a>) -> Option<&'a mut Self> {
        if Self::Is_valid_pointer(Pointer) {
            return None;
        }

        Some(&mut *Pointer)
    }

    pub unsafe fn From_mutable_pointer_to_box(
        Pointer: *mut Raw_mutex_type<'a>,
    ) -> Option<Box<Self>> {
        if Self::Is_valid_pointer(Pointer) {
            return None;
        }

        Some(Box::from_raw(Pointer))
    }

    pub fn Lock(&'a mut self) -> bool {
        // If the mutex is recursive, we can lock it multiple times from the same thread.
        if self.Recursive {
            if let Some(Metadata) = self.Metadata.as_ref() {
                if Metadata.Thread == std::thread::current().id() {
                    return true;
                }
            }
        }

        // If the mutex is not recursive or the current thread is not the owner, we need to lock it.
        let Guard = match self.Mutex.lock() {
            Ok(Guard) => Guard,
            Err(_) => return false,
        };

        self.Metadata
            .replace(Metadata_type {
                Guard,
                Thread: std::thread::current().id(),
            })
            .is_some()
    }

    pub fn Unlock(&mut self) -> bool {
        match self.Metadata.as_ref() {
            Some(Metadata) => {
                if Metadata.Thread != std::thread::current().id() {
                    return false;
                };
            }
            None => {
                return false;
            }
        }

        self.Metadata.take();

        true // Guard is dropped here
    }
}
