#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![no_std]

mod Allocator;
mod Cache;
mod Capability;
mod Protection;
mod Region_allocator;
mod Statistics;

pub use Allocator::*;
pub use Cache::*;
pub use Capability::*;
pub use Protection::*;
pub use Region_allocator::*;
pub use Statistics::*;

pub type Layout_type = core::alloc::Layout;
