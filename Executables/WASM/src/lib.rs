#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

mod error;
mod main;

pub use error::*;
use executable::Implement_executable_device;
pub use main::*;

pub struct WASM_device_type;

Implement_executable_device!(
    Structure: WASM_device_type,
    Mount_path: "/Binaries/WASM",
    Main_function: Main::main,
);
