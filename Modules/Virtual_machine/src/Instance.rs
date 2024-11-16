#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{
    ffi::{c_void, CString},
    ptr::null_mut,
};

use wamr_rust_sdk::{
    function::Function,
    instance::Instance,
    sys::{
        wasm_runtime_addr_native_to_app, wasm_runtime_set_wasi_args_ex,
        wasm_runtime_validate_native_addr,
    },
    value::WasmValue,
};
use File_system::Unique_file_identifier_type;

use crate::{
    Custom_data_type, Error_type, Module::Module_type, Result_type, Runtime::Runtime_type,
    WASM_pointer_type,
};

pub struct Instance_type {
    Instance: Instance,
    _Directory_paths: Vec<CString>,
    _Directory_paths_raw: Vec<*const i8>,
    _Environment_variables_raw: Vec<*const i8>,
unsafe impl Send for Instance_type {}

impl Drop for Instance_type {
    fn drop(&mut self) {
        let Instance = self.Get_inner_reference().get_inner_instance();
        unsafe {
            let User_data = wasm_runtime_get_custom_data(Instance) as *mut Custom_data_type;

            if !User_data.is_null() {
                let _ = Box::from_raw(User_data);
            }
        }

        // User data is dropped here.
    }
}

impl Instance_type {
    pub fn New(
        Runtime: &Runtime_type,
        Module: &Module_type,
        Stack_size: usize,
    ) -> Result_type<Self> {
        let WAMR_instance = Instance::new(
            Runtime.Get_inner_reference(),
            Module.Get_inner_reference(),
            Stack_size as u32,
        )?;

        let Instance = Instance_type {
            Instance: WAMR_instance,
            _Directory_paths: Directory_paths,
            _Directory_paths_raw: Directory_paths_raw,
            _Environment_variables_raw: Environment_variables_raw,
        };

        Ok(Instance)
    }

    pub fn Validate_native_pointer<T>(&self, Pointer: *const T, Size: usize) -> bool {
        unsafe {
            wasm_runtime_validate_native_addr(
                self.Get_inner_reference().get_inner_instance(),
                Pointer as *mut c_void,
                Size as u64,
            )
        }
    }

    pub fn Validate_WASM_pointer(&self, Address: WASM_pointer_type, Size: usize) -> bool {
        unsafe {
            wasm_runtime_validate_native_addr(
                self.Get_inner_reference().get_inner_instance(),
                Address as *mut c_void,
                Size as u64,
            )
        }
    }

    pub fn Convert_to_WASM_pointer<T>(&self, Pointer: *const T) -> WASM_pointer_type {
        unsafe {
            wasm_runtime_addr_native_to_app(
                self.Get_inner_reference().get_inner_instance(),
                Pointer as *mut c_void,
            ) as WASM_pointer_type
        }
    }

    pub fn Convert_to_native_pointer<T>(&self, Address: WASM_pointer_type) -> *mut T {
        unsafe {
            wasm_runtime_addr_native_to_app(
                self.Get_inner_reference().get_inner_instance(),
                Address as *mut c_void,
            ) as *mut T
        }
    }

    pub fn Call_export_function(
        &self,
        Name: &str,
        Parameters: &Vec<WasmValue>,
    ) -> Result_type<WasmValue> {
        if Parameters.is_empty() {
            Ok(
                Function::find_export_func(self.Get_inner_reference(), Name)?
                    .call(&self.Instance, &vec![WasmValue::I32(0)])?,
            )
        } else {
            Ok(
                Function::find_export_func(self.Get_inner_reference(), Name)?
                    .call(&self.Instance, Parameters)?,
            )
        }
    }

    pub fn Call_main(&self, Parameters: &Vec<WasmValue>) -> Result_type<WasmValue> {
        self.Call_export_function("_start", Parameters)
    }

    pub(crate) fn Get_inner_reference(&self) -> &Instance {
        &self.Instance
    }
}
