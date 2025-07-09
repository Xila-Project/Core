#![allow(non_camel_case_types)]

extern crate alloc;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
extern crate std;

#[cfg(target_vendor = "espressif")]
pub mod espressif;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub mod native;

pub mod core;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub mod standard_library;

//pub fn Mount_devices(
//    Task: Task_identifier_type,
//    Virtual_file_systems: &Virtual_file_system_type,
//) -> Result<(), String> {
//    #[cfg(target_vendor = "espressif")]
//    Espressif::Mount_devices(Virtual_file_systems)?;
//
//    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
//    Native::Mount_devices(Task, Virtual_file_systems)?;
//
//    Ok(())
//}
