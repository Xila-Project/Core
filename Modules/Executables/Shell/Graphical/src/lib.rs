#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use Executable::Standard_type;

mod Desk;
mod Device;
mod Main;

pub use Device::*;

pub struct Shell_type {
    Standard: Standard_type,
    User: String,
    Running: bool,
}
