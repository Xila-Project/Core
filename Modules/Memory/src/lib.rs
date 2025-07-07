#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

extern crate alloc;

mod Cache;
mod Capability;
mod Macro;
mod Manager;
mod Protection;
mod Statistics;
mod Trait;

pub use Cache::*;
pub use Capability::*;
pub use Manager::*;
pub use Protection::*;
pub use Statistics::*;
pub use Trait::*;

pub type Layout_type = core::alloc::Layout;
