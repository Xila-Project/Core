use core::ffi::c_void;

use xila::shared::BijectiveBTreeMap;

use crate::host::virtual_machine::WasmPointer;

#[derive(Debug, Clone, Default)]
pub struct CustomData {
    pub translation_map: BijectiveBTreeMap<WasmPointer, *mut c_void>,
}
