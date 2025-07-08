#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;
use file_system::Create_device;
use log::Information;
use shared::Duration_type;

#[test]
fn test_get_current_time() {
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::new()));

    let Current_time = Time::get_instance().get_current_time().unwrap();

    Information!("Current time : {Current_time:?}");

    assert_ne!(Current_time, Duration_type::default());
}

#[test]
fn test_get_current_time_since_startup() {
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::new()));

    let Current_time = Time::get_instance()
        .get_current_time_since_startup()
        .unwrap();

    Information!("Time since startup : {Current_time:?}");

    assert_ne!(Current_time, Duration_type::default());
}
