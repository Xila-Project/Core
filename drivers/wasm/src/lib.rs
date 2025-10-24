#![cfg(target_arch = "wasm32")]
#![no_std]

extern crate alloc;

pub mod devices;
pub mod executor;
pub mod log;
pub mod memory;

pub extern crate memory as memory_exported;
