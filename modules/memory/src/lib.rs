#![no_std]

extern crate alloc;

mod cache;
mod capability;
mod r#macro;
mod manager;
mod protection;
mod statistics;
mod r#trait;

pub use cache::*;
pub use capability::*;
pub use manager::*;
pub use protection::*;
pub use r#trait::*;
pub use statistics::*;

pub type Layout = core::alloc::Layout;
