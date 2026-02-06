use core::ffi::c_void;

use crate::host::virtual_machine::{CustomData, Environment, WasmPointer};

#[derive(Clone, Copy)]
pub struct Translator<'a> {
    environment: &'a Environment,
    custom_data: &'a CustomData,
}

impl<'a> Translator<'a> {
    pub fn new(environment: &'a Environment, custom_data: &'a CustomData) -> Translator<'a> {
        Translator {
            environment,
            custom_data,
        }
    }

    pub unsafe fn translate_to_host<T>(
        &self,
        wasm_address: WasmPointer,
        owned_by_guest: bool,
    ) -> Option<*mut T> {
        if owned_by_guest {
            unsafe { self.environment.translate_to_host(wasm_address) }
        } else {
            self.custom_data
                .translation_map
                .get_by_left(&wasm_address)
                .copied()
        }
    }

    pub unsafe fn translate_to_guest<T>(
        &self,
        host_address: *mut T,
        owned_by_guest: bool,
    ) -> Option<WasmPointer> {
        if owned_by_guest {
            unsafe { self.environment.translate_to_guest(host_address) }
        } else {
            self.custom_data
                .translation_map
                .get_by_right(&host_address)
                .copied()
        }
    }
}
