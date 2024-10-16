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
    Data::Data_type, Environment_type, Error_type, Module::Module_type, Result_type,
    Runtime::Runtime_type, WASM_pointer_type,
};

pub struct Instance_type {
    Instance: Instance,
    _Directory_paths: Vec<CString>,
    _Directory_paths_raw: Vec<*const i8>,
    _Environment_variables_raw: Vec<*const i8>,
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
        // - Directory paths allowed to be accessed by the WASI module.
        let Directory_paths = vec![CString::new("/").unwrap()];

        let mut Directory_paths_raw: Vec<*const i8> =
            Directory_paths.iter().map(|x| x.as_ptr()).collect();

        // - Environment variables.
        let Task_instance = Task::Get_instance();

        let Task = Task_instance
            .Get_current_task_identifier()
            .map_err(Error_type::Failed_to_get_task_informations)?;

        let mut Environment_variables_raw: Vec<*const i8> = Task_instance
            .Get_environment_variables(Task)
            .map_err(Error_type::Failed_to_get_task_informations)?
            .into_iter()
            .map(|x| x.Get_raw().as_ptr())
            .collect();

        unsafe {
            wasm_runtime_set_wasi_args_ex(
                Module.Get_inner_reference().get_inner_module(),
                Directory_paths_raw.as_mut_ptr(),
                Directory_paths_raw.len() as u32,
                null_mut(),
                0,
                Environment_variables_raw.as_mut_ptr(),
                Environment_variables_raw.len() as u32,
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
            _Directory_paths: Directory_paths,
            _Directory_paths_raw: Directory_paths_raw,
            _Environment_variables_raw: Environment_variables_raw,
        };

        let mut Execution_environment = Environment_type::From_instance(&Instance)?;

        Execution_environment.Set_user_data(Data);

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
