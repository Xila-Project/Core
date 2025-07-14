#![no_std]
#![allow(clippy::useless_transmute)]

#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
