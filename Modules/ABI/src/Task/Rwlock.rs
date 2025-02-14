use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
enum Guard_type<'a> {
    #[allow(dead_code)]
    Read(RwLockReadGuard<'a, ()>),
    #[allow(dead_code)]
    Write(RwLockWriteGuard<'a, ()>),
}

#[derive(Debug)]
pub struct Raw_rwlock_type<'a> {
    Lock: RwLock<()>,
    #[allow(dead_code)]
    Guard: Option<Guard_type<'a>>,
}

impl<'a> Raw_rwlock_type<'a> {
    pub fn New() -> Self {
        Self {
            Lock: RwLock::new(()),
            Guard: None,
        }
    }

    pub fn Is_valid_pointer(Pointer: *const Raw_rwlock_type<'a>) -> bool {
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
    ///  This function may return an error if the pointer is null or not aligned.
    pub unsafe fn From_pointer(Pointer: *const Raw_rwlock_type<'a>) -> Option<&'a Self> {
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
    pub unsafe fn From_mutable_pointer(Pointer: *mut Raw_rwlock_type<'a>) -> Option<&'a mut Self> {
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
        Pointer: *mut Raw_rwlock_type<'a>,
    ) -> Option<Box<Self>> {
        if Self::Is_valid_pointer(Pointer) {
            return None;
        }

        Some(Box::from_raw(Pointer))
    }

    pub fn Read(&'a mut self) -> bool {
        if self.Guard.is_some() {
            return false;
        }

        match self.Lock.read() {
            Ok(Guard) => {
                self.Guard = Some(Guard_type::Read(Guard));
                true
            }
            Err(_) => false,
        }
    }

    pub fn Write(&'a mut self) -> bool {
        if self.Guard.is_some() {
            return false;
        }

        match self.Lock.write() {
            Ok(Guard) => {
                self.Guard = Some(Guard_type::Write(Guard));
                true
            }
            Err(_) => false,
        }
    }

    pub fn Unlock(&mut self) -> bool {
        match self.Guard {
            Some(_) => (),
            None => return false,
        }

        self.Guard = None;

        true
    }
}

/// This function is used to initialize a rwlock.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the rwlock is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_initialize_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    if Rwlock.is_null() {
        return false;
    }

    if Rwlock as usize % std::mem::align_of::<Raw_rwlock_type>() != 0 {
        return false;
    }

    unsafe {
        Rwlock.write(Raw_rwlock_type::New());
    }

    true
}

/// This function is used to read a rwlock.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the rwlock is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_read_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    let Rwlock = match Raw_rwlock_type::From_mutable_pointer(Rwlock) {
        Some(Rwlock) => Rwlock,
        None => return false,
    };

    Rwlock.Read()
}

/// This function is used to write a rwlock.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the rwlock is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_write_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    let Rwlock = match Raw_rwlock_type::From_mutable_pointer(Rwlock) {
        Some(Rwlock) => Rwlock,
        None => return false,
    };

    Rwlock.Write()
}

/// This function is used to unlock a rwlock.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the rwlock is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_unlock_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    let Rwlock = match Raw_rwlock_type::From_mutable_pointer(Rwlock) {
        Some(Rwlock) => Rwlock,
        None => return false,
    };

    Rwlock.Unlock()
}

/// This function is used to destroy a rwlock.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the rwlock is not initialized.
#[no_mangle]
pub unsafe extern "C" fn Xila_destroy_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    let _ = match Raw_rwlock_type::From_mutable_pointer_to_box(Rwlock) {
        Some(RwLock) => RwLock,
        None => return false,
    };

    true // RwLock is dropped here
}
