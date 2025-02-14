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

    /// Transforms a pointer to a reference.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    ///
    /// # Errors
    ///
    /// This function may return an error if the pointer is null or not aligned.
    pub unsafe fn From_pointer(Pointer: *const Raw_mutex_type<'a>) -> Option<&'a Self> {
        if Self::Is_valid_pointer(Pointer) {
            return None;
        }

        Some(&*Pointer)
    }

    /// Transforms a mutable pointer to a mutable reference.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    ///
    /// # Errors
    ///
    /// This function may return an error if the pointer is null or not aligned.
    pub unsafe fn From_mutable_pointer(Pointer: *mut Raw_mutex_type<'a>) -> Option<&'a mut Self> {
        if Self::Is_valid_pointer(Pointer) {
            return None;
        }

        Some(&mut *Pointer)
    }

    /// Transforms a mutable pointer to a box.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    ///
    /// # Errors
    ///
    /// This function may return an error if the pointer is null or not aligned.
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

#[no_mangle]
pub static Raw_mutex_size: usize = size_of::<Raw_mutex_type>();

/// This function is used to initialize a mutex.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the mutex is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_initialize_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    if Mutex.is_null() {
        return false;
    }

    if Mutex as usize % std::mem::align_of::<Raw_mutex_type>() != 0 {
        return false;
    }

    Mutex.write(Raw_mutex_type::New(false));

    true
}

/// # Safety
///
///
///
#[no_mangle]
pub unsafe extern "C" fn Xila_initialize_recursive_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    if Mutex.is_null() {
        return false;
    }

    if Mutex as usize % std::mem::align_of::<Raw_mutex_type>() != 0 {
        return false;
    }

    Mutex.write(Raw_mutex_type::New(true));

    true
}

/// This function is used to lock a mutex.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the mutex is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_lock_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    let Mutex = match Raw_mutex_type::From_mutable_pointer(Mutex) {
        Some(Mutex) => Mutex,
        None => return false,
    };

    Mutex.Lock()
}

/// This function is used to unlock a mutex.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the mutex is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_unlock_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    let Mutex = match Raw_mutex_type::From_mutable_pointer(Mutex) {
        Some(Mutex) => Mutex,
        None => return false,
    };

    Mutex.Unlock()
}

/// This function is used to destroy a mutex.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors    
///
/// This function may return an error if the mutex is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_destroy_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    let _ = match Raw_mutex_type::From_mutable_pointer_to_box(Mutex) {
        Some(Mutex) => Mutex,
        None => return false,
    };

    true // Mutex is dropped here
}
