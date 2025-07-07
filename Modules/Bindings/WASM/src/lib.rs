#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::useless_transmute)]

#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/Bindings.rs"));
