use crate::host::virtual_machine::{Result, WasmPointer};
use alloc::boxed::Box;
use core::ffi::c_void;
use wamr_rust_sdk::sys::{
    WASMExecEnv, wasm_exec_env_t, wasm_module_inst_t, wasm_runtime_addr_app_to_native,
    wasm_runtime_addr_native_to_app, wasm_runtime_get_custom_data, wasm_runtime_get_module_inst,
    wasm_runtime_set_custom_data,
};

pub type EnvironmentPointer = wasm_exec_env_t;

#[repr(transparent)]
pub struct Environment(WASMExecEnv);

impl Environment {
    pub unsafe fn from_raw_pointer<'a>(raw_pointer: *mut WASMExecEnv) -> &'a mut Self {
        unsafe { &mut *(raw_pointer as *mut Self) }
    }

    pub unsafe fn get_or_initialize_custom_data<'b, T: Default>(&mut self) -> Result<&'b mut T> {
        unsafe {
            let custom_data = wasm_runtime_get_custom_data(self.get_instance_pointer()) as *mut T;

            let custom_data = if custom_data.is_null() {
                let custom_data = Box::new(T::default());

                wasm_runtime_set_custom_data(
                    self.get_instance_pointer(),
                    Box::into_raw(custom_data) as *mut c_void,
                );

                wasm_runtime_get_custom_data(self.get_instance_pointer()) as *mut T
            } else {
                custom_data
            };

            Ok(&mut *custom_data)
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn translate_to_host<T>(&mut self, address: WasmPointer) -> Option<*mut T> {
        unsafe {
            let pointer =
                wasm_runtime_addr_app_to_native(self.get_instance_pointer(), address as u64);

            if pointer.is_null() {
                return None;
            }

            if !(pointer as usize).is_multiple_of(core::mem::align_of::<T>()) {
                return None;
            }

            Some(pointer as *mut T)
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn translate_to_guest<T>(&mut self, pointer: *mut T) -> WasmPointer {
        unsafe {
            wasm_runtime_addr_native_to_app(self.get_instance_pointer(), pointer as *mut c_void)
                as WasmPointer
        }
    }

    fn get_instance_pointer(&mut self) -> wasm_module_inst_t {
        unsafe { wasm_runtime_get_module_inst(&mut self.0) }
    }
}
