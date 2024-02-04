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

use wamr_sys::{wasm_runtime_destroy, wasm_runtime_init};

pub fn Initialize() -> Result<(), ()> {
    unsafe {
        if !wasm_runtime_init() {
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
