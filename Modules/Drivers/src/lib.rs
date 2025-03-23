#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;

#[cfg(target_vendor = "espressif")]
pub mod Espressif;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub mod Native;

pub mod Core;

#[cfg(feature = "std")]
pub mod Std;

pub fn Mount_devices(
    Task: Task_identifier_type,
    Virtual_file_systems: &Virtual_file_system_type,
) -> Result<(), String> {
    #[cfg(target_vendor = "espressif")]
    Espressif::Mount_devices(Virtual_file_systems)?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    Native::Mount_devices(Task, Virtual_file_systems)?;

    Ok(())
}
