mod Function;
mod Module;
pub mod Prelude;
mod Symbol;

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
