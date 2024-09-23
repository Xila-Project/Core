#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[cfg(target_os = "espidf")]
mod Espressif;
mod Flags;
mod Layout;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod Native;

mod Protection;
mod Trait;

pub use Flags::*;
pub use Layout::*;
pub use Protection::*;
pub use Trait::*;

#[cfg(target_os = "espidf")]
pub static Allocator: Espressif::Memory_allocator_type = Espressif::Memory_allocator_type::New();

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub static Allocator: Native::Memory_allocator_type = Native::Memory_allocator_type::New();
