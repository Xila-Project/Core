use core::ffi::c_void;

use crate::host::{
    bindings::graphics::error::{Error, Result},
    virtual_machine::{Environment, WasmPointer, WasmUsize},
};
use alloc::collections::btree_map::{BTreeMap, Entry};
use xila::task::TaskIdentifier;

pub(crate) struct TranslationMap {
    to_native_pointer: BTreeMap<usize, *mut c_void>,
    to_wasm_pointer: BTreeMap<*mut c_void, u16>,
}

impl TranslationMap {
    pub fn new() -> Self {
        Self {
            to_native_pointer: BTreeMap::new(),
            to_wasm_pointer: BTreeMap::new(),
        }
    }

    const fn get_identifier(task: TaskIdentifier, identifier: u16) -> usize {
        (task.into_inner() as usize) << 32 | identifier as usize
    }

    pub fn insert(&mut self, task: TaskIdentifier, pointer: *mut c_void) -> Result<u16> {
        for i in u16::MIN..u16::MAX {
            let identifier = Self::get_identifier(task, i);

            match self.to_native_pointer.entry(identifier) {
                Entry::Vacant(entry) => {
                    entry.insert(pointer);
                    self.to_wasm_pointer.insert(pointer, i);
                    return Ok(i);
                }
                Entry::Occupied(entry_pointer) => {
                    if *entry_pointer.get() == pointer {
                        return Ok(i);
                    }
                }
            }
        }

        Err(Error::PointerTableFull)
    }

    pub fn get_native_pointer<T>(
        &self,
        task: TaskIdentifier,
        identifier: WasmUsize,
    ) -> Result<*mut T> {
        let identifier = Self::get_identifier(task, identifier);

        self.to_native_pointer
            .get(&identifier)
            .map(|pointer| *pointer as *mut T)
            .ok_or(Error::NativePointerNotFound)
    }

    pub fn get_wasm_pointer<T>(&self, pointer: *mut T) -> Result<u16> {
        self.to_wasm_pointer
            .get(&(pointer as *mut c_void))
            .cloned()
            .ok_or(Error::WasmPointerNotFound)
    }

    pub fn remove<T>(&mut self, task: TaskIdentifier, identifier: u16) -> Result<*mut T> {
        let identifier = Self::get_identifier(task, identifier);

        let pointer = self
            .to_native_pointer
            .remove(&identifier)
            .map(|pointer| pointer as *mut T)
            .ok_or(Error::NativePointerNotFound)?;

        self.to_wasm_pointer.remove(&(pointer as *mut _));

        Ok(pointer)
    }
}

pub unsafe fn translate_to_native_pointer<T>(
    environment: &Environment,
    pointer: WasmPointer,
) -> Result<*mut T> {
    unsafe {
        environment
            .translate_to_host_pointer(pointer)
            .ok_or(Error::InvalidPointer)
    }
}
