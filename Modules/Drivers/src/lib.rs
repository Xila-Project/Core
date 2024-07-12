#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

#[cfg(target_vendor = "espressif")]
pub mod Espressif;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub mod Native;
