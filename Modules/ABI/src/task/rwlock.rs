use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
};
use synchronization::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};

pub struct Raw_rwlock_type {
    /// Mutex to protect the lock state.
    ///
    /// The lock state is represented as follows:
    /// - 0: Unlocked
    /// - 1: Write locked (no readers allowed)
    /// - 2 or more: Read locked (number of readers)
    mutex: Mutex<CriticalSectionRawMutex, usize>,
}

impl Raw_rwlock_type {
    const READING: usize = 2; // Represents the state when there are readers
    const WRITING: usize = 1; // Represents the state when there is a writer
    const UNLOCKED: usize = 0; // Represents the state when the rwlock is unlocked

    pub fn new() -> Self {
        Self {
            mutex: Mutex::new(Self::UNLOCKED), // Initial state: unlocked
        }
    }

    pub fn is_valid_pointer(pointer: *const Raw_rwlock_type) -> bool {
        !pointer.is_null() && (pointer as usize % align_of::<Self>() == 0)
    }

    /// Transforms a pointer to a reference.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    /// The caller must ensure the pointer is valid and points to properly initialized memory.
    pub unsafe fn from_pointer<'a>(pointer: *const Raw_rwlock_type) -> Option<&'a Self> {
        if !Self::is_valid_pointer(pointer) {
            return None;
        }
        Some(&*pointer)
    }

    /// Transforms a mutable pointer to a mutable reference.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    /// The caller must ensure the pointer is valid and points to properly initialized memory.
    pub unsafe fn from_mutable_pointer<'a>(pointer: *mut Raw_rwlock_type) -> Option<&'a mut Self> {
        if !Self::is_valid_pointer(pointer) {
            return None;
        }
        Some(&mut *pointer)
    }

    pub fn read(&self) -> bool {
        unsafe {
            self.mutex.lock_mut(|state| {
                // Can't read if there's a writer (state == 1)

                match *state {
                    Self::WRITING => return false, // Write lock prevents reading
                    Self::UNLOCKED => *state = Self::READING, // Unlocked, can read
                    _ => *state += 1,              // Already has readers, can add more
                }

                true
            })
        }
    }

    pub fn write(&self) -> bool {
        unsafe {
            self.mutex.lock_mut(|state| {
                // Can only write if completely unlocked
                if *state != Self::UNLOCKED {
                    return false;
                }

                *state = Self::WRITING; // Write lock
                true
            })
        }
    }

    pub fn unlock(&self) -> bool {
        unsafe {
            self.mutex.lock_mut(|state| {
                match *state {
                    Self::UNLOCKED => false, // Not locked
                    Self::WRITING => {
                        // Write lock - unlock completely
                        *state = Self::UNLOCKED;
                        true
                    }
                    n if n >= 2 => {
                        // Read lock - decrement reader count
                        *state -= 1;
                        if *state == Self::WRITING {
                            // This shouldn't happen, but fix it
                            *state = Self::UNLOCKED;
                        }
                        true
                    }
                    _ => false,
                }
            })
        }
    }
}

#[no_mangle]
pub static RAW_RWLOCK_SIZE: usize = size_of::<Raw_rwlock_type>();

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
pub unsafe extern "C" fn Xila_initialize_rwlock(rwlock: *mut Raw_rwlock_type) -> bool {
    if rwlock.is_null() {
        return false;
    }

    if rwlock as usize % align_of::<Raw_rwlock_type>() != 0 {
        return false;
    }

    rwlock.write(Raw_rwlock_type::new());

    true
}

/// Read lock a rwlock.
///
/// # Safety
///
/// The caller must ensure:
/// - `rwlock` points to a valid, initialized `Raw_rwlock_type`
/// - The rwlock remains valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn Xila_read_rwlock(rwlock: *mut Raw_rwlock_type) -> bool {
    let rwlock = match Raw_rwlock_type::from_mutable_pointer(rwlock) {
        Some(rwlock) => rwlock,
        None => return false,
    };

    rwlock.read()
}

/// Write lock a rwlock.
///
/// # Safety
///
/// The caller must ensure:
/// - `rwlock` points to a valid, initialized `Raw_rwlock_type`
/// - The rwlock remains valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn Xila_write_rwlock(rwlock: *mut Raw_rwlock_type) -> bool {
    let rwlock = match Raw_rwlock_type::from_mutable_pointer(rwlock) {
        Some(rwlock) => rwlock,
        None => return false,
    };

    rwlock.write()
}

/// Unlock a rwlock.
///
/// # Safety
///
/// The caller must ensure:
/// - `rwlock` points to a valid, initialized `Raw_rwlock_type`
/// - The rwlock remains valid for the duration of the call
/// - The current task owns the lock (either read or write)
#[no_mangle]
pub unsafe extern "C" fn Xila_unlock_rwlock(rwlock: *mut Raw_rwlock_type) -> bool {
    let rwlock = match Raw_rwlock_type::from_mutable_pointer(rwlock) {
        Some(rwlock) => rwlock,
        None => return false,
    };

    rwlock.unlock()
}

/// Destroy a rwlock.
///
/// # Safety
///
/// The caller must ensure:
/// - `rwlock` points to a valid, initialized `Raw_rwlock_type` allocated with Box
/// - The rwlock is not currently locked
/// - No other threads are waiting on the rwlock
#[no_mangle]
pub unsafe extern "C" fn Xila_destroy_rwlock(rwlock: *mut Raw_rwlock_type) -> bool {
    let _ = match Raw_rwlock_type::from_mutable_pointer(rwlock) {
        Some(rw_lock) => rw_lock,
        None => return false,
    };

    drop_in_place(rwlock); // Drop the rwlock, releasing resources

    true // RwLock is dropped here
}
