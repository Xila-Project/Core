#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod Device;
mod Error;
mod File_system;
mod Fundamentals;

mod Memory_device;
mod Time;

pub use Device::{Device_trait, Device_type};
pub use Error::*;

pub use File_system::*;
pub use Fundamentals::*;
pub use Memory_device::*;
pub use Time::*;
