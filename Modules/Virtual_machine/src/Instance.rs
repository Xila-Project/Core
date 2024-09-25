#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{ffi::CString, mem::forget, ptr::null_mut};

use wamr_rust_sdk::{
    function::Function, instance::Instance, sys::wasm_runtime_set_wasi_args_ex, value::WasmValue,
};
use File_system::Unique_file_identifier_type;

use crate::{
    Data::Data_type, Environment_type, Module::Module_type, Result_type, Runtime::Runtime_type,
};

pub struct Instance_type {
    Instance: Instance,
    Directory_paths_raw: Vec<*const i8>,
}

impl Drop for Instance_type {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .Directory_paths_raw
                .iter_mut()
                .map(|x| CString::from_raw(*x as *mut i8))
                .collect::<Vec<CString>>();

            // Directory_paths will be dropped here and paths memory will be freed.
        }
    }
}

impl Instance_type {
    pub fn New(
        Runtime: &Runtime_type,
        Module: &Module_type,
        Stack_size: usize,
        Data: &Data_type,
        Standard_in: Unique_file_identifier_type,
        Standard_out: Unique_file_identifier_type,
        Standard_error: Unique_file_identifier_type,
    ) -> Result_type<Self> {
        let mut Directory_paths = [CString::new("/").unwrap()];

        let mut Directory_paths_raw: Vec<*const i8> =
            Directory_paths.iter().map(|x| x.as_ptr()).collect();

        Directory_paths.iter_mut().for_each(forget);

        unsafe {
            wasm_runtime_set_wasi_args_ex(
                Module.Get_inner_reference().get_inner_module(),
                Directory_paths_raw.as_mut_ptr(),
                Directory_paths_raw.len() as u32,
                null_mut(),
                0,
                null_mut(),
                0,
                null_mut(),
                0,
                std::mem::transmute::<Unique_file_identifier_type, i64>(Standard_in),
                std::mem::transmute::<Unique_file_identifier_type, i64>(Standard_out),
                std::mem::transmute::<Unique_file_identifier_type, i64>(Standard_error),
            )
        }

        let WAMR_instance = Instance::new(
            Runtime.Get_inner_reference(),
            Module.Get_inner_reference(),
            Stack_size as u32,
        )?;

        let Instance = Instance_type {
            Instance: WAMR_instance,
            Directory_paths_raw,
        };

        let mut Execution_environment = Environment_type::From_instance(&Instance)?;

        Execution_environment.Set_user_data(Data);

        Ok(Instance)
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
