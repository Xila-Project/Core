#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;

mod Error;
mod Main;

pub use Error::*;
use Executable::Implement_executable_device;
pub use Main::*;

pub struct WASM_device_type;

Implement_executable_device!(
    Structure: WASM_device_type,
    Mount_path: "/Binaries/WASM",
    Main_function: Main::Main,
);
