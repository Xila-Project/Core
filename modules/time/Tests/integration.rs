#![no_std]

extern crate alloc;
use core::time::Duration;
use file_system::create_device;
use log::information;

#[test]
fn test_get_current_time() {
    let _ = time::initialize(create_device!(drivers::native::TimeDriver::new()));

    let current_time = time::get_instance().get_current_time().unwrap();

    information!("Current time : {current_time:?}");

    assert_ne!(current_time, Duration::default());
}

#[test]
fn test_get_current_time_since_startup() {
    let _ = time::initialize(create_device!(drivers::native::TimeDriver::new()));

    let current_time = time::get_instance()
        .get_current_time_since_startup()
        .unwrap();

    information!("Time since startup : {current_time:?}");

    assert_ne!(current_time, Duration::default());
}
