use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
};
use Synchronization::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};

use crate::Context;

#[derive(Debug, Clone, Copy, Default)]
struct Mutex_state_type {
    task: Option<usize>,
    lock_count: u32, // For recursive mutexes
}

pub struct Raw_mutex_type {
    mutex: Mutex<CriticalSectionRawMutex, Mutex_state_type>,
    recursive: bool,
}

impl Raw_mutex_type {
    pub fn new(recursive: bool) -> Self {
        Self {
            mutex: Mutex::new(Mutex_state_type::default()),
            recursive,
        }
    }

    pub fn is_valid_pointer(pointer: *const Raw_mutex_type) -> bool {
        !pointer.is_null() && (pointer as usize % align_of::<Self>() == 0)
    }

    /// Transforms a pointer to a reference.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    /// The caller must ensure the pointer is valid and points to properly initialized memory.
    pub unsafe fn From_pointer<'a>(pointer: *const Raw_mutex_type) -> Option<&'a Self> {
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
    pub unsafe fn From_mutable_pointer<'a>(pointer: *mut Raw_mutex_type) -> Option<&'a mut Self> {
        if !Self::is_valid_pointer(pointer) {
            return None;
        }
        Some(&mut *pointer)
    }

    pub fn Lock(&self) -> bool {
        let current_task = Context::get_instance()
            .get_current_task_identifier()
            .Into_inner() as usize;

        unsafe {
            self.mutex.lock_mut(|State| {
                if let Some(owner) = State.task {
                    if owner == current_task && self.recursive {
                        // Recursive lock
                        State.lock_count += 1;
                        return true;
                    }
                    // Mutex is already locked by another task
                    return false;
                }

                // Lock is available
                State.task = Some(current_task);
                State.lock_count = 1;
                true
            })
        }
    }

    pub fn unlock(&self) -> bool {
        let current_task = Context::get_instance()
            .get_current_task_identifier()
            .Into_inner() as usize;

        unsafe {
            self.mutex.lock_mut(|state| {
                // Check if current task owns the mutex
                if let Some(owner) = state.task {
                    if owner == current_task {
                        if self.recursive && state.lock_count > 1 {
                            // Decrement lock count for recursive mutex
                            state.lock_count -= 1;
                        } else {
                            // Unlock the mutex
                            state.task = None;
                            state.lock_count = 0;
                        }
                        return true; // Successfully unlocked
                    }
                }
                false // Not owned by current task or not locked
            })
        }
    }
}

#[no_mangle]
pub static RAW_MUTEX_SIZE: usize = size_of::<Raw_mutex_type>();

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

    if Mutex as usize % align_of::<Raw_mutex_type>() != 0 {
        return false;
    }

    Mutex.write(Raw_mutex_type::new(false));

    true
}

/// Initialize a recursive mutex.
///
/// # Safety
///
/// The caller must ensure:
/// - `mutex` points to valid, uninitialized memory
/// - The memory is properly aligned for `Raw_mutex_type`
/// - The memory will remain valid for the lifetime of the mutex
#[no_mangle]
pub unsafe extern "C" fn Xila_initialize_recursive_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    if Mutex.is_null() {
        return false;
    }

    if Mutex as usize % align_of::<Raw_mutex_type>() != 0 {
        return false;
    }

    Mutex.write(Raw_mutex_type::new(true));

    true
}

/// Lock a mutex (blocking).
///
/// # Safety
///
/// The caller must ensure:
/// - `mutex` points to a valid, initialized `Raw_mutex_type`
/// - The mutex remains valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn Xila_lock_mutex(mutex: *mut Raw_mutex_type) -> bool {
    let mutex = match Raw_mutex_type::From_mutable_pointer(mutex) {
        Some(mutex) => mutex,
        None => return false,
    };

    mutex.Lock()
}

/// Unlock a mutex (blocking).
///
/// # Safety
///
/// The caller must ensure:
/// - `mutex` points to a valid, initialized `Raw_mutex_type`
/// - The mutex remains valid for the duration of the call
/// - The current task owns the mutex
#[no_mangle]
pub unsafe extern "C" fn Xila_unlock_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    let mutex = match Raw_mutex_type::From_mutable_pointer(Mutex) {
        Some(mutex) => mutex,
        None => return false,
    };

    mutex.unlock()
}

/// Destroy a mutex.
///
/// # Safety
///
/// The caller must ensure:
/// - `mutex` points to a valid, initialized `Raw_mutex_type` allocated with Box
/// - The mutex is not currently locked
/// - No other threads are waiting on the mutex
#[no_mangle]
pub unsafe extern "C" fn Xila_destroy_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    let mutex = match Raw_mutex_type::From_mutable_pointer(Mutex) {
        Some(mutex) => mutex,
        None => return false,
    };

    // Drop the mutex, which will release any resources it holds
    drop_in_place(mutex);

    true // Mutex is dropped here
}
