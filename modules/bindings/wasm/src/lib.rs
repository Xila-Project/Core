#![no_std]
#![allow(clippy::useless_transmute)]

#[cfg(target_arch = "wasm32")]
mod prelude;

#[cfg(target_arch = "wasm32")]
pub use prelude::*;

#[cfg(target_arch = "wasm32")]
mod enumeration;

#[cfg(target_arch = "wasm32")]
use enumeration::*;

#[cfg(target_arch = "wasm32")]
mod functions;

#[cfg(target_arch = "wasm32")]
pub use functions::*;

#[cfg(all(target_arch = "wasm32", feature = "c_bindings"))]
pub mod c_functions;
