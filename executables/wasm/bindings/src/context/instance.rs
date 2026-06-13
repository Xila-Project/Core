use core::ffi::c_void;
use core::ptr::NonNull;

unsafe extern "Rust" {
    pub unsafe fn __wasm_get_instance_context() -> Option<NonNull<InstanceContext>>;
}

use xila::shared::BijectiveBTreeMap;

use crate::WasmPointer;

#[derive(Debug, Clone, Default)]
pub struct InstanceContext {
    pub translation_map: BijectiveBTreeMap<WasmPointer, *mut c_void>,
}

impl InstanceContext {
    pub fn new() -> Self {
        Self {
            translation_map: BijectiveBTreeMap::new(),
        }
    }
}
