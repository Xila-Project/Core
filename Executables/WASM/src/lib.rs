#![no_std]
#![allow(non_camel_case_types)]

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
    Main_function: Main::main,
);
