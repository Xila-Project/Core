#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

mod cache;
mod capability;
mod macro;
mod manager;
mod protection;
mod statistics;
mod trait;

pub use cache::*;
pub use capability::*;
pub use manager::*;
pub use protection::*;
pub use statistics::*;
pub use trait::*;

pub type Layout_type = core::alloc::Layout;
