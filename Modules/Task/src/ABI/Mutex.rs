use crate::Raw_mutex::Raw_mutex_type;

#[no_mangle]
pub static Raw_mutex_size: usize = size_of::<Raw_mutex_type>();

#[no_mangle]
pub unsafe extern "C" fn Xila_initialize_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    println!("Initializing mutex : {:p}", Mutex);

    if Mutex.is_null() {
        return false;
    }

    if Mutex as usize % std::mem::align_of::<Raw_mutex_type>() != 0 {
        return false;
    }

    Mutex.write(Raw_mutex_type::New(false));

    println!("Mutex initialized : {:?}", unsafe { Mutex.read() });

    true
}

/// # Safety
///
///
///
#[no_mangle]
pub unsafe extern "C" fn Xila_initialize_recursive_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    println!("Initializing recursive mutex");

    if Mutex.is_null() {
        return false;
    }

    if Mutex as usize % std::mem::align_of::<Raw_mutex_type>() != 0 {
        return false;
    }

    Mutex.write(Raw_mutex_type::New(true));

    true
}

#[no_mangle]
pub unsafe extern "C" fn Xila_lock_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    println!(
        "Locking mutex : {:?} : {:p}",
        unsafe { Mutex.read() },
        Mutex
    );

    let Mutex = match Raw_mutex_type::From_mutable_pointer(Mutex) {
        Some(Mutex) => Mutex,
        None => return false,
    };

    Mutex.Lock()
}

#[no_mangle]
pub unsafe extern "C" fn Xila_unlock_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    println!(
        "Unlocking mutex : {:?} : {:p}",
        unsafe { Mutex.read() },
        Mutex
    );

    let Mutex = match Raw_mutex_type::From_mutable_pointer(Mutex) {
        Some(Mutex) => Mutex,
        None => return false,
    };

    Mutex.Unlock()
}

#[no_mangle]
pub unsafe extern "C" fn Xila_destroy_mutex(Mutex: *mut Raw_mutex_type) -> bool {
    println!(
        "Destroying mutex : {:?} : {:p}",
        unsafe { Mutex.read() },
        Mutex
    );

    let _ = match Raw_mutex_type::From_mutable_pointer_to_box(Mutex) {
        Some(Mutex) => Mutex,
        None => return false,
    };

    true // Mutex is dropped here
}
