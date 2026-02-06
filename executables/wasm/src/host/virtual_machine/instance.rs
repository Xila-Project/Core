use core::ffi::c_void;

use alloc::{boxed::Box, vec, vec::Vec};
use wamr_rust_sdk::{
    function::Function,
    instance,
    sys::{
        wasm_runtime_addr_app_to_native, wasm_runtime_addr_native_to_app,
        wasm_runtime_get_custom_data, wasm_runtime_validate_native_addr,
    },
    value::WasmValue,
};

use crate::host::virtual_machine::{
    CustomData, Error, Result, WasmPointer, module::Module, runtime::Runtime,
};

pub struct Instance<'module> {
    instance: instance::Instance<'module>,
}

unsafe impl Send for Instance<'_> {}

impl Drop for Instance<'_> {
    fn drop(&mut self) {
        let instance = self.get_inner_reference().get_inner_instance();
        unsafe {
            let user_data = wasm_runtime_get_custom_data(instance) as *mut CustomData;

            if !user_data.is_null() {
                let _ = Box::from_raw(user_data);
            }
        }

        // User data is dropped here.
    }
}

impl<'module> Instance<'module> {
    pub fn new(
        runtime: &Runtime,
        module: &'module Module<'module>,
        stack_size: usize,
    ) -> Result<Self> {
        let wamr_instance = instance::Instance::new(
            runtime.get_inner_reference(),
            module.get_inner_reference(),
            stack_size as u32,
        )?;

        let instance = Instance {
            instance: wamr_instance,
        };

        Ok(instance)
    }

    pub fn validate_native_pointer<T>(&self, pointer: *const T, size: usize) -> bool {
        unsafe {
            wasm_runtime_validate_native_addr(
                self.get_inner_reference().get_inner_instance(),
                pointer as *mut c_void,
                size as u64,
            )
        }
    }

    pub fn validate_wasm_pointer(&self, address: WasmPointer, size: usize) -> bool {
        unsafe {
            wasm_runtime_validate_native_addr(
                self.get_inner_reference().get_inner_instance(),
                address as *mut c_void,
                size as u64,
            )
        }
    }

    pub fn translate_to_guest_pointer<T>(&self, pointer: *const T) -> WasmPointer {
        unsafe {
            wasm_runtime_addr_native_to_app(
                self.get_inner_reference().get_inner_instance(),
                pointer as *mut c_void,
            ) as WasmPointer
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn translate_to_host_pointer<T>(&self, address: WasmPointer) -> *mut T {
        unsafe {
            wasm_runtime_addr_app_to_native(
                self.get_inner_reference().get_inner_instance(),
                address as u64,
            ) as *mut T
        }
    }

    pub fn call_exported_function(
        &self,
        name: &str,
        parameters: &Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>> {
        if parameters.is_empty() {
            Ok(
                Function::find_export_func(self.get_inner_reference(), name)?
                    .call(&self.instance, &vec![WasmValue::I32(0)])?,
            )
        } else {
            Ok(
                Function::find_export_func(self.get_inner_reference(), name)?
                    .call(&self.instance, parameters)?,
            )
        }
    }

    pub fn get_inner_reference(&'_ self) -> &'_ instance::Instance<'_> {
        &self.instance
    }
}
