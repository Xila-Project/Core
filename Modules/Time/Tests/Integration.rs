#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;
use File_system::Create_device;
use Log::Information;
use Shared::Duration_type;

#[test]
fn Test_get_current_time() {
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    let Current_time = Time::Get_instance().Get_current_time().unwrap();

    Information!("Current time : {Current_time:?}");

    assert_ne!(Current_time, Duration_type::default());
}

#[test]
fn Test_get_current_time_since_startup() {
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    let Current_time = Time::Get_instance()
        .Get_current_time_since_startup()
        .unwrap();

    Information!("Time since startup : {Current_time:?}");

    assert_ne!(Current_time, Duration_type::default());
}
