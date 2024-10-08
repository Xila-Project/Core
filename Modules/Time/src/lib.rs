#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Error;
mod Manager;

pub use Error::*;
pub use Manager::*;

pub type Duration_type = core::time::Duration;
