#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

#[cfg(target_vendor = "espressif")]
pub mod Espressif;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub mod Native;

pub mod Time;

pub fn Mount_file_systems(
    Virtual_file_systems: &File_system::Virtual_file_system_type,
) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    Native::Mount_file_systems(Virtual_file_systems)?;

    Ok(())
}

pub fn Mount_devices(
    Virtual_file_systems: &File_system::Virtual_file_system_type,
) -> Result<(), String> {
    #[cfg(target_vendor = "espressif")]
    Espressif::Mount_devices(Virtual_file_systems)?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    Native::Mount_devices(Virtual_file_systems)?;

    Virtual_file_systems.Add_device(&"/Device/Time", Box::new(Time::Time_device_type));

    Ok(())
}
