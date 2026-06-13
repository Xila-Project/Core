use core::ffi::c_void;

use wasm_abi_bindings::WasmPointer;

use crate::host::virtual_machine::{EnvironmentReference, InstanceReference, Result};

pub struct Translator<'a> {
    instance: &'a mut InstanceReference,
}

impl<'a> Translator<'a> {
    pub unsafe fn from_environment(environment: &'a EnvironmentReference) -> Result<Self> {
        let instance = environment.get_instance();
        Ok(Self { instance })
    }

    pub fn add_host_translation<T>(&mut self, host_address: *mut T) -> WasmPointer {
        let context = self.instance.get_context();

        let guest_address = {
            let mut next_id = 1; // Start from your preferred minimum (0 or 1)

            for &id in context.translation_map.get_left_keys() {
                if id > next_id {
                    // We found a gap!
                    return next_id;
                }
                next_id = id + 1;
            }

            next_id
        };

        context
            .translation_map
            .insert(guest_address, host_address as *mut c_void);

        guest_address
    }

    pub fn remove_host_translation<T>(&mut self, guest_address: WasmPointer) -> Option<*mut T> {
        self.instance
            .get_context()
            .translation_map
            .remove_by_key(&guest_address)
            .map(|ptr| ptr as *mut T)
    }

    pub unsafe fn translate_to_host<T>(
        &mut self,
        wasm_address: WasmPointer,
        owned_by_guest: bool,
    ) -> Option<*mut T> {
        if owned_by_guest {
            unsafe { self.instance.translate_to_host(wasm_address) }
        } else {
            self.instance
                .get_context()
                .translation_map
                .get_by_left(&wasm_address)
                .copied()
                .map(|ptr| ptr as *mut T)
        }
    }

    pub unsafe fn translate_to_guest<T>(
        &mut self,
        host_address: *mut T,
        owned_by_guest: bool,
    ) -> Option<WasmPointer> {
        if owned_by_guest {
            unsafe { Some(self.instance.translate_to_guest(host_address)) }
        } else {
            self.instance
                .get_context()
                .translation_map
                .get_by_right(&(host_address as *mut c_void))
                .copied()
        }
    }
}
