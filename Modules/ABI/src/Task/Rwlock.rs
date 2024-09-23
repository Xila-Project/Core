use Task::Raw_rwlock::Raw_rwlock_type;

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
