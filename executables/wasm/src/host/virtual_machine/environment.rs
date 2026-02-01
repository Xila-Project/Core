use core::{
    ffi::{CStr, c_void},
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
use xila::abi_context;

use crate::host::virtual_machine::{CustomData, Error, Instance, Result, WasmPointer, WasmUsize};

pub type EnvironmentPointer = wasm_exec_env_t;

#[derive(Debug, Clone, Copy)]
pub struct Environment<'a>(EnvironmentPointer, PhantomData<&'a ()>);

unsafe impl Send for Environment<'_> {}

unsafe impl Sync for Environment<'_> {}

impl Environment<'_> {
    pub fn from_raw_pointer(raw_pointer: EnvironmentPointer) -> Result<Self> {
        if raw_pointer.is_null() {
            return Err(Error::InvalidPointer);
        }

        Ok(Self(raw_pointer as EnvironmentPointer, PhantomData))
    }

    pub fn from_instance(instance: &Instance) -> Result<Self> {
        let instance_pointer = instance.get_inner_reference().get_inner_instance();

        if instance_pointer.is_null() {
            return Err(Error::InvalidPointer);
        }
        Ok(Self(
            unsafe { wasm_runtime_get_exec_env_singleton(instance_pointer) },
            PhantomData,
        ))
    }

    pub fn get_or_initialize_custom_data(&self) -> Result<&CustomData> {
        unsafe {
            let custom_data =
                wasm_runtime_get_custom_data(self.get_instance_pointer()) as *const CustomData;

            let custom_data = if custom_data.is_null() {
                let task = abi_context::get_instance().get_current_task_identifier();

                let custom_data = Box::new(CustomData::new(task));

                wasm_runtime_set_custom_data(
                    self.get_instance_pointer(),
                    Box::into_raw(custom_data) as *mut c_void,
                );

                wasm_runtime_get_custom_data(self.get_instance_pointer()) as *const CustomData
            } else {
                custom_data
            };

            Ok(&*custom_data)
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn translate_to_host_pointer<T>(&self, address: WasmPointer) -> Option<*mut T> {
        unsafe {
            let pointer =
                wasm_runtime_addr_app_to_native(self.get_instance_pointer(), address as u64);

            if pointer.is_null() {
                return None;
            }

            Some(pointer as *mut T)
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn translate_to_wasm_pointer<T>(&self, pointer: *mut T) -> WasmPointer {
        unsafe {
            wasm_runtime_addr_native_to_app(self.get_instance_pointer(), pointer as *mut c_void)
                as WasmPointer
        }
    }

    pub fn validate_wasm_pointer(&self, address: WasmPointer, size: WasmUsize) -> bool {
        unsafe {
            wasm_runtime_validate_app_addr(self.get_instance_pointer(), address as u64, size as u64)
        }
    }

    pub fn validate_native_pointer<T>(&self, pointer: *const T, size: u64) -> bool {
        unsafe {
            wasm_runtime_validate_native_addr(
                self.get_instance_pointer(),
                pointer as *mut c_void,
                size,
            )
        }
    }

    /// Make an indirect function call (call a function by its index which is not exported).
    /// For exported functions use `Call_export_function`.
    pub fn call_indirect_function(
        &self,
        function_index: u32,
        parameters: &Vec<WasmValue>,
    ) -> Result<()> {
        let mut arguments = Vec::new();

        for parameter in parameters {
            arguments.append(&mut parameter.encode());
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
            let exception_message =
                unsafe { wasm_runtime_get_exception(self.get_instance_pointer()) };
            let exception_message = unsafe { CStr::from_ptr(exception_message) };
            let exception_message =
                String::from_utf8_lossy(exception_message.to_bytes()).to_string();

            return Err(Error::ExecutionError(exception_message));
        }

        Ok(())
    }

    /// Create a new execution environment.
    /// This environment should be initialized with `Initialize_thread_environment` and deinitialized with `Deinitialize_thread_environment`.
    pub fn create_environment(&self, stack_size: usize) -> Result<Self> {
        let execution_environment =
            unsafe { wasm_runtime_create_exec_env(self.get_instance_pointer(), stack_size as u32) };

        if execution_environment.is_null() {
            return Err(Error::ExecutionError(
                "Execution environment creation failed".to_string(),
            ));
        }

        Ok(Self(execution_environment, PhantomData))
    }

    //    pub fn set_instruction_count_limit(&self, limit: Option<u64>) {
    //        unsafe {
    //            wasm_runtime_set_instruction_count_limit(
    //                self.get_inner_reference(),
    //                limit.map(|limit| limit as i32).unwrap_or(-1),
    //            );
    //        }
    //    }

    fn get_instance_pointer(&self) -> wasm_module_inst_t {
        unsafe { wasm_runtime_get_module_inst(self.0) }
    }

    #[allow(dead_code)]
    pub(crate) fn get_inner_reference(&self) -> EnvironmentPointer {
        self.0
    }
}
