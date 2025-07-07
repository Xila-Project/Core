#![allow(non_camel_case_types)]

use core::{
    ffi::{c_void, CStr},
    marker::PhantomData,
};

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use wamr_rust_sdk::{
    sys::{
        wasm_exec_env_t, wasm_module_inst_t, wasm_runtime_addr_app_to_native,
        wasm_runtime_addr_native_to_app, wasm_runtime_call_indirect, wasm_runtime_create_exec_env,
        wasm_runtime_get_custom_data, wasm_runtime_get_exception,
        wasm_runtime_get_exec_env_singleton, wasm_runtime_get_module_inst,
        wasm_runtime_set_custom_data, wasm_runtime_set_instruction_count_limit,
        wasm_runtime_validate_app_addr, wasm_runtime_validate_native_addr,
    },
    value::WasmValue,
};

use crate::{
    Custom_data_type, Error_type, Instance_type, Result_type, WASM_pointer_type, WASM_usize_type,
};

pub type Environment_pointer_type = wasm_exec_env_t;

#[derive(Debug, Clone, Copy)]
pub struct Environment_type<'a>(Environment_pointer_type, PhantomData<&'a ()>);

unsafe impl Send for Environment_type<'_> {}

unsafe impl Sync for Environment_type<'_> {}

impl Environment_type<'_> {
    pub fn from_raw_pointer(Raw_pointer: Environment_pointer_type) -> Result_type<Self> {
        if Raw_pointer.is_null() {
            return Err(Error_type::Invalid_pointer);
        }

        Ok(Self(Raw_pointer as Environment_pointer_type, PhantomData))
    }

    pub fn From_instance(Instance: &Instance_type) -> Result_type<Self> {
        let instance_pointer = Instance.Get_inner_reference().get_inner_instance();

        if instance_pointer.is_null() {
            return Err(Error_type::Invalid_pointer);
        }
        Ok(Self(
            unsafe { wasm_runtime_get_exec_env_singleton(instance_pointer) },
            PhantomData,
        ))
    }

    pub fn Get_or_initialize_custom_data(&self) -> Result_type<&Custom_data_type> {
        unsafe {
            let custom_data = wasm_runtime_get_custom_data(self.Get_instance_pointer())
                as *const Custom_data_type;

            let Custom_data = if custom_data.is_null() {
                let task = ABI::Get_instance().Get_current_task_identifier();

                let Custom_data = Box::new(Custom_data_type::new(task));

                wasm_runtime_set_custom_data(
                    self.Get_instance_pointer(),
                    Box::into_raw(Custom_data) as *mut c_void,
                );

                wasm_runtime_get_custom_data(self.Get_instance_pointer()) as *const Custom_data_type
            } else {
                custom_data
            };

            Ok(&*Custom_data)
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn Convert_to_native_pointer<T>(
        &self,
        address: WASM_pointer_type,
    ) -> Option<*mut T> {
        let pointer = wasm_runtime_addr_app_to_native(self.Get_instance_pointer(), address as u64);

        if pointer.is_null() {
            return None;
        }

        Some(pointer as *mut T)
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn Convert_to_WASM_pointer<T>(&self, Pointer: *const T) -> WASM_pointer_type {
        wasm_runtime_addr_native_to_app(self.Get_instance_pointer(), Pointer as *mut c_void)
            as WASM_pointer_type
    }

    pub fn Validate_WASM_pointer(&self, Address: WASM_pointer_type, Size: WASM_usize_type) -> bool {
        unsafe {
            wasm_runtime_validate_app_addr(self.Get_instance_pointer(), Address as u64, Size as u64)
        }
    }

    pub fn Validate_native_pointer<T>(&self, Pointer: *const T, Size: u64) -> bool {
        unsafe {
            wasm_runtime_validate_native_addr(
                self.Get_instance_pointer(),
                Pointer as *mut c_void,
                Size,
            )
        }
    }

    /// Make an indirect function call (call a function by its index which is not exported).
    /// For exported functions use `Call_export_function`.
    pub fn Call_indirect_function(
        &self,
        function_index: u32,
        parameters: &Vec<WasmValue>,
    ) -> Result_type<()> {
        let mut arguments = Vec::new();

        for Parameter in parameters {
            arguments.append(&mut Parameter.encode());
        }

        if arguments.is_empty() {
            arguments.append(&mut WasmValue::I32(0).encode());
        }

        if !unsafe {
            wasm_runtime_call_indirect(
                self.0,
                function_index,
                arguments.len() as u32,
                arguments.as_mut_ptr(),
            )
        } {
            let Exception_message =
                unsafe { wasm_runtime_get_exception(self.Get_instance_pointer()) };
            let exception_message = unsafe { CStr::from_ptr(Exception_message) };
            let exception_message =
                String::from_utf8_lossy(exception_message.to_bytes()).to_string();

            return Err(Error_type::Execution_error(exception_message));
        }

        Ok(())
    }

    /// Create a new execution environment.
    /// This environment should be initialized with `Initialize_thread_environment` and deinitialized with `Deinitialize_thread_environment`.
    pub fn Create_environment(&self, Stack_size: usize) -> Result_type<Self> {
        let execution_environment =
            unsafe { wasm_runtime_create_exec_env(self.Get_instance_pointer(), Stack_size as u32) };

        if execution_environment.is_null() {
            return Err(Error_type::Execution_error(
                "Execution environment creation failed".to_string(),
            ));
        }

        Ok(Self(execution_environment, PhantomData))
    }

    pub fn Set_instruction_count_limit(&self, Limit: Option<u64>) {
        unsafe {
            wasm_runtime_set_instruction_count_limit(
                self.get_inner_reference(),
                Limit.map(|limit| limit as i32).unwrap_or(-1),
            );
        }
    }

    fn Get_instance_pointer(&self) -> wasm_module_inst_t {
        unsafe { wasm_runtime_get_module_inst(self.0) }
    }

    #[allow(dead_code)]
    pub(crate) fn get_inner_reference(&self) -> Environment_pointer_type {
        self.0
    }
}
