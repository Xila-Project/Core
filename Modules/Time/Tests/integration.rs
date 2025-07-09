#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;
use file_system::create_device;
use log::Information;
use shared::Duration_type;

#[test]
fn test_get_current_time() {
    let _ = time::initialize(create_device!(drivers::native::Time_driver_type::new()));

    let current_time = time::get_instance().get_current_time().unwrap();

    Information!("Current time : {current_time:?}");

    assert_ne!(current_time, Duration_type::default());
}

#[test]
fn test_get_current_time_since_startup() {
    let _ = time::initialize(create_device!(drivers::native::Time_driver_type::new()));

    let current_time = time::get_instance()
        .get_current_time_since_startup()
        .unwrap();

    Information!("Time since startup : {current_time:?}");

    assert_ne!(current_time, Duration_type::default());
}
