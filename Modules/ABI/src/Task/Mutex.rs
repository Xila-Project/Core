use Task::Raw_mutex::Raw_mutex_type;

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
