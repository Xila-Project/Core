#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod Fundamentals;

pub use Fundamentals::*;

mod Function;
pub use Function::*;

mod Module;
pub use Module::*;

pub mod Prelude;
pub use Prelude::*;

mod Symbol;
pub use Symbol::*;

mod Memory;
pub use Memory::*;

#[macro_export]
macro_rules! Declare_native_symbol {
    ($name:ident, $signature:expr) => {
        paste::paste! {
            pub const [< $name _symbol >] : super::Native_symbol_type = super::Native_symbol_type {
                symbol: concat!(stringify!($name), "\0").as_ptr() as *const i8,
                func_ptr: super::$name as *mut _,
                signature: concat!($signature, "\0").as_ptr() as *const i8,
                attachment: std::ptr::null_mut(),
            };
        }
    };
}

use wamr_sys::{wasm_runtime_destroy, wasm_runtime_init, wasm_runtime_register_natives};

pub fn Initialize() -> Result<(), ()> {
    unsafe {
        if !wasm_runtime_init() {
            return Err(());
        }
    }
    Ok(())
}

const Environment: &str = "env\0";

pub fn Register_native_symbols(Symbols: &mut [Native_symbol_type]) -> Result<(), ()> {
    unsafe {
        if !wasm_runtime_register_natives(
            Environment.as_ptr() as *const _,
            Symbols.as_mut_ptr() as *mut _,
            Symbols.len() as u32,
        ) {
            return Err(());
        }
    }
    Ok(())
}

pub fn Destroy() {
    unsafe {
        wasm_runtime_destroy();
    }
}
