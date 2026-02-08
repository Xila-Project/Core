use core::ffi::c_void;

use crate::host::virtual_machine::{CustomData, Environment, Result, WasmPointer};

pub struct Translator<'a> {
    environment: &'a mut Environment,
    custom_data: &'a mut CustomData,
}

impl<'a> Translator<'a> {
    pub unsafe fn from_environment(environment: &'a mut Environment) -> Result<Self> {
        let custom_data = unsafe { environment.get_or_initialize_custom_data()? };

        Ok(Self {
            environment,
            custom_data,
        })
    }

    pub fn add_host_translation<T>(&mut self, host_address: *mut T) -> WasmPointer {
        let guest_address = {
            let mut next_id = 1; // Start from your preferred minimum (0 or 1)

            for &id in self.custom_data.translation_map.get_left_keys() {
                if id > next_id {
                    // We found a gap!
                    return next_id;
                }
                next_id = id + 1;
            }

            next_id
        };

        self.custom_data
            .translation_map
            .insert(guest_address, host_address as *mut c_void);

        guest_address
    }

    pub fn remove_host_translation<T>(&mut self, guest_address: WasmPointer) -> Option<*mut T> {
        self.custom_data
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
            unsafe { self.environment.translate_to_host(wasm_address) }
        } else {
            self.custom_data
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
            unsafe { Some(self.environment.translate_to_guest(host_address)) }
        } else {
            self.custom_data
                .translation_map
                .get_by_right(&(host_address as *mut c_void))
                .copied()
        }
    }
}
