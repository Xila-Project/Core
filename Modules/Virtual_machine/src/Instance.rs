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

pub struct Instance_type {
    Instance: Instance,
    Allocate: Option<Function>,
    Deallocate: Option<Function>,
}

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

        let Allocate = Function::find_export_func(&WAMR_instance, "Allocate").ok();

        let Deallocate = Function::find_export_func(&WAMR_instance, "Deallocate").ok();

        let Instance = Instance_type {
            Instance: WAMR_instance,
            Allocate,
            Deallocate,
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

    pub fn Allocate<T>(&mut self, Size: usize) -> Result_type<*mut T> {
        let Function = self
            .Allocate
            .as_ref()
            .ok_or(Error_type::Allocation_failure)?;

        let Pointer = Function.call(&self.Instance, &vec![WasmValue::I32(Size as i32)])?;

        if let WasmValue::I32(Pointer) = Pointer {
            let Pointer = unsafe { self.Convert_to_native_pointer(Pointer as u32) };

            Ok(Pointer)
        } else {
            Err(Error_type::Allocation_failure)
        }
    }

    pub fn Deallocate<T>(&mut self, Data: *mut T) {
        let Function = self
            .Deallocate
            .as_ref()
            .ok_or(Error_type::Allocation_failure)
            .expect("No deallocate function found in exports");

        let Pointer = self.Convert_to_WASM_pointer(Data);

        Function
            .call(&self.Instance, &vec![WasmValue::I32(Pointer as i32)])
            .expect("Failed to deallocate pointer");
    }

    pub(crate) fn Get_inner_reference(&self) -> &Instance {
        &self.Instance
    }
}
