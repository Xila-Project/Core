#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[cfg(all(target_arch = "wasm32", feature = "WASM"))]
pub use WASM_bindings::*;
