use core::ffi::c_void;

unsafe extern "Rust" {
    pub unsafe fn __wasm_get_instance_data() -> *mut InstanceContext;
}

use xila::{shared::BijectiveBTreeMap, task::TaskIdentifier};

use crate::WasmPointer;

#[derive(Debug, Clone, Default)]
pub struct InstanceContext {
    pub translation_map: BijectiveBTreeMap<WasmPointer, *mut c_void>,
}

impl InstanceContext {
    pub fn new(task: TaskIdentifier) -> Self {
        Self {
            translation_map: BijectiveBTreeMap::new(),
        }
    }
}
