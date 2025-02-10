#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::c_void;

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
    Custom_data_type, Error_type, Module::Module_type, Result_type, Runtime::Runtime_type,
    WASM_pointer_type,
};

pub struct Instance_type<'module> {
    Instance: Instance<'module>,
}

unsafe impl Send for Instance_type<'_> {}

impl Drop for Instance_type<'_> {
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

impl<'module> Instance_type<'module> {
    pub fn New(
        Runtime: &Runtime_type,
        Module: &'module Module_type<'module>,
        Stack_size: usize,
    ) -> Result_type<Self> {
        let WAMR_instance = Instance::new(
            Runtime.Get_inner_reference(),
            Module.Get_inner_reference(),
            Stack_size as u32,
        )?;

        let Instance = Instance_type {
            Instance: WAMR_instance,
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

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn Convert_to_native_pointer<T>(&self, Address: WASM_pointer_type) -> *mut T {
        wasm_runtime_addr_app_to_native(
            self.Get_inner_reference().get_inner_instance(),
            Address as u64,
        ) as *mut T
    }

    pub fn Call_export_function(
        &self,
        Name: &str,
        Parameters: &Vec<WasmValue>,
    ) -> Result_type<Vec<WasmValue>> {
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

    pub fn Call_main(&self, Parameters: &Vec<WasmValue>) -> Result_type<Vec<WasmValue>> {
        self.Call_export_function("_start", Parameters)
    }

    pub fn Allocate<T>(&mut self, Size: usize) -> Result_type<*mut T> {
        let Result = self.Call_export_function("Allocate", &vec![WasmValue::I32(Size as i32)])?;

        if let Some(WasmValue::I32(Pointer)) = Result.first() {
            let Pointer = unsafe { self.Convert_to_native_pointer(*Pointer as u32) };

            Ok(Pointer)
        } else {
            Err(Error_type::Allocation_failure)
        }
    }

    pub fn Deallocate<T>(&mut self, Data: *mut T) {
        let _ = self.Call_export_function("Deallocate", &vec![WasmValue::I32(Data as i32)]);
    }

    pub(crate) fn Get_inner_reference(&self) -> &Instance {
        &self.Instance
    }
}
