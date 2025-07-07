use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
};
use Synchronization::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};

pub struct Raw_rwlock_type {
    /// Mutex to protect the lock state.
    ///
    /// The lock state is represented as follows:
    /// - 0: Unlocked
    /// - 1: Write locked (no readers allowed)
    /// - 2 or more: Read locked (number of readers)
    Mutex: Mutex<CriticalSectionRawMutex, usize>,
}

impl Raw_rwlock_type {
    const READING: usize = 2; // Represents the state when there are readers
    const WRITING: usize = 1; // Represents the state when there is a writer
    const UNLOCKED: usize = 0; // Represents the state when the rwlock is unlocked

    pub fn New() -> Self {
        Self {
            Mutex: Mutex::new(Self::UNLOCKED), // Initial state: unlocked
        }
    }

    pub fn Is_valid_pointer(pointer: *const Raw_rwlock_type) -> bool {
        !pointer.is_null() && (pointer as usize % align_of::<Self>() == 0)
    }

    /// Transforms a pointer to a reference.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    /// The caller must ensure the pointer is valid and points to properly initialized memory.
    pub unsafe fn From_pointer<'a>(pointer: *const Raw_rwlock_type) -> Option<&'a Self> {
        if !Self::Is_valid_pointer(pointer) {
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
    pub unsafe fn From_mutable_pointer<'a>(pointer: *mut Raw_rwlock_type) -> Option<&'a mut Self> {
        if !Self::Is_valid_pointer(pointer) {
            return None;
        }
        Some(&mut *pointer)
    }

    pub fn Read(&self) -> bool {
        unsafe {
            self.Mutex.lock_mut(|State| {
                // Can't read if there's a writer (state == 1)

                match *State {
                    Self::WRITING => return false, // Write lock prevents reading
                    Self::UNLOCKED => *State = Self::READING, // Unlocked, can read
                    _ => *State += 1,              // Already has readers, can add more
                }

                true
            })
        }
    }

    pub fn Write(&self) -> bool {
        unsafe {
            self.Mutex.lock_mut(|State| {
                // Can only write if completely unlocked
                if *State != Self::UNLOCKED {
                    return false;
                }

                *State = Self::WRITING; // Write lock
                true
            })
        }
    }

    pub fn Unlock(&self) -> bool {
        unsafe {
            self.Mutex.lock_mut(|State| {
                match *State {
                    Self::UNLOCKED => false, // Not locked
                    Self::WRITING => {
                        // Write lock - unlock completely
                        *State = Self::UNLOCKED;
                        true
                    }
                    n if n >= 2 => {
                        // Read lock - decrement reader count
                        *State -= 1;
                        if *State == Self::WRITING {
                            // This shouldn't happen, but fix it
                            *State = Self::UNLOCKED;
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
pub unsafe extern "C" fn Xila_initialize_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    if Rwlock.is_null() {
        return false;
    }

    if Rwlock as usize % align_of::<Raw_rwlock_type>() != 0 {
        return false;
    }

    Rwlock.write(Raw_rwlock_type::New());

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
pub unsafe extern "C" fn Xila_read_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    let Rwlock = match Raw_rwlock_type::From_mutable_pointer(Rwlock) {
        Some(Rwlock) => Rwlock,
        None => return false,
    };

    Rwlock.Read()
}

/// Write lock a rwlock.
///
/// # Safety
///
/// The caller must ensure:
/// - `rwlock` points to a valid, initialized `Raw_rwlock_type`
/// - The rwlock remains valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn Xila_write_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    let Rwlock = match Raw_rwlock_type::From_mutable_pointer(Rwlock) {
        Some(Rwlock) => Rwlock,
        None => return false,
    };

    Rwlock.Write()
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
pub unsafe extern "C" fn Xila_unlock_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    let Rwlock = match Raw_rwlock_type::From_mutable_pointer(Rwlock) {
        Some(Rwlock) => Rwlock,
        None => return false,
    };

    Rwlock.Unlock()
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
pub unsafe extern "C" fn Xila_destroy_rwlock(Rwlock: *mut Raw_rwlock_type) -> bool {
    let _ = match Raw_rwlock_type::From_mutable_pointer(Rwlock) {
        Some(RwLock) => RwLock,
        None => return false,
    };

    drop_in_place(Rwlock); // Drop the rwlock, releasing resources

    true // RwLock is dropped here
}
