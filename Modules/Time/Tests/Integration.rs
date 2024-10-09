#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use File_system::Create_device;
use Shared::Duration_type;

#[test]
fn Test_get_current_time() {
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    let Current_time = Time::Get_instance().Get_current_time().unwrap();

    println!("Current time : {:?}", Current_time);

    assert_ne!(Current_time, Duration_type::default());
}

#[test]
fn Test_get_current_time_since_startup() {
    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    let Current_time = Time::Get_instance()
        .Get_current_time_since_startup()
        .unwrap();

    println!("Time since startup : {:?}", Current_time);

    assert_ne!(Current_time, Duration_type::default());
}
