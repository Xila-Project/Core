#![allow(non_camel_case_types)]

use core::ffi::c_void;

use alloc::{boxed::Box, vec, vec::Vec};
use wamr_rust_sdk::{
    function::Function,
    instance::Instance,
    sys::{
        wasm_runtime_addr_app_to_native, wasm_runtime_addr_native_to_app,
        wasm_runtime_get_custom_data, wasm_runtime_validate_native_addr,
    },
    value::WasmValue,
};

use crate::{
    module::Module_type, runtime::Runtime_type, Custom_data_type, Error_type, Result_type,
    WASM_pointer_type,
};

pub struct Instance_type<'module> {
    instance: Instance<'module>,
}

unsafe impl Send for Instance_type<'_> {}

impl Drop for Instance_type<'_> {
    fn drop(&mut self) {
        let instance = self.get_inner_reference().get_inner_instance();
        unsafe {
            let user_data = wasm_runtime_get_custom_data(instance) as *mut Custom_data_type;

            if !user_data.is_null() {
                let _ = Box::from_raw(user_data);
            }
        }

        // User data is dropped here.
    }
}

impl<'module> Instance_type<'module> {
    pub fn new(
        runtime: &Runtime_type,
        module: &'module Module_type<'module>,
        stack_size: usize,
    ) -> Result_type<Self> {
        let wamr_instance = Instance::new(
            runtime.get_inner_reference(),
            module.get_inner_reference(),
            stack_size as u32,
        )?;

        let instance = Instance_type {
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

    pub fn validate_wasm_pointer(&self, address: WASM_pointer_type, size: usize) -> bool {
        unsafe {
            wasm_runtime_validate_native_addr(
                self.get_inner_reference().get_inner_instance(),
                address as *mut c_void,
                size as u64,
            )
        }
    }

    pub fn convert_to_wasm_pointer<T>(&self, pointer: *const T) -> WASM_pointer_type {
        unsafe {
            wasm_runtime_addr_native_to_app(
                self.get_inner_reference().get_inner_instance(),
                pointer as *mut c_void,
            ) as WASM_pointer_type
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn convert_to_native_pointer<T>(&self, address: WASM_pointer_type) -> *mut T {
        wasm_runtime_addr_app_to_native(
            self.get_inner_reference().get_inner_instance(),
            address as u64,
        ) as *mut T
    }

    pub fn call_export_function(
        &self,
        name: &str,
        parameters: &Vec<WasmValue>,
    ) -> Result_type<Vec<WasmValue>> {
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

    pub fn call_main(&self, parameters: &Vec<WasmValue>) -> Result_type<Vec<WasmValue>> {
        self.call_export_function("_start", parameters)
    }

    pub fn allocate<T>(&mut self, size: usize) -> Result_type<*mut T> {
        let result = self.call_export_function("Allocate", &vec![WasmValue::I32(size as i32)])?;

        if let Some(WasmValue::I32(pointer)) = result.first() {
            let pointer = unsafe { self.convert_to_native_pointer(*pointer as u32) };

            Ok(pointer)
        } else {
            Err(Error_type::Allocation_failure)
        }
    }

    pub fn deallocate<T>(&mut self, data: *mut T) {
        let _ = self.call_export_function("Deallocate", &vec![WasmValue::I32(data as i32)]);
    }

    pub fn get_inner_reference(&self) -> &Instance {
        &self.instance
    }
}
