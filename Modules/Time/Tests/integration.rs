#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;
use file_system::Create_device;
use log::Information;
use shared::Duration_type;

#[test]
fn test_get_current_time() {
    let _ = time::initialize(Create_device!(drivers::native::Time_driver_type::new()));

    let Current_time = time::get_instance().get_current_time().unwrap();

    Information!("Current time : {Current_time:?}");

    assert_ne!(Current_time, Duration_type::default());
}

#[test]
fn test_get_current_time_since_startup() {
    let _ = time::initialize(Create_device!(drivers::native::Time_driver_type::new()));

    let Current_time = time::get_instance()
        .get_current_time_since_startup()
        .unwrap();

    Information!("Time since startup : {Current_time:?}");

    assert_ne!(Current_time, Duration_type::default());
}
