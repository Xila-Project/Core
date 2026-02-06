use core::ffi::c_void;

use xila::{shared::BijectiveBTreeMap, task::TaskIdentifier};

use crate::host::virtual_machine::WasmPointer;

#[derive(Debug, Clone)]
pub struct CustomData {
    pub task_identifier: TaskIdentifier,
    pub translation_map: BijectiveBTreeMap<WasmPointer, *mut c_void>,
}
