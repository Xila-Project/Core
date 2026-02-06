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
        WASMExecEnv, wasm_exec_env_t, wasm_module_inst_t, wasm_runtime_addr_app_to_native,
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

#[repr(transparent)]
pub struct Environment(WASMExecEnv);

impl Environment {
    pub unsafe fn from_raw_pointer<'a>(raw_pointer: *mut WASMExecEnv) -> &'a Self {
        unsafe { &*(raw_pointer as *const Self) }
    }

    pub fn from_instance<'a>(instance: &Instance) -> Result<&'a Self> {
        let instance_pointer = instance.get_inner_reference().get_inner_instance();

        let envionment_pointer = unsafe { wasm_runtime_get_exec_env_singleton(instance_pointer) };

        Ok(unsafe { Self::from_raw_pointer(envionment_pointer) })
    }

    pub fn get_or_initialize_custom_data<T: Default>(&self) -> Result<&T> {
        unsafe {
            let custom_data = wasm_runtime_get_custom_data(self.get_instance_pointer()) as *const T;

            let custom_data = if custom_data.is_null() {
                let task = abi_context::get_instance().get_current_task_identifier();

                let custom_data = Box::new(T::default());

                wasm_runtime_set_custom_data(
                    self.get_instance_pointer(),
                    Box::into_raw(custom_data) as *mut c_void,
                );

                wasm_runtime_get_custom_data(self.get_instance_pointer()) as *const T
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
    pub unsafe fn translate_to_host<T>(&self, address: WasmPointer) -> Option<*mut T> {
        unsafe {
            let pointer =
                wasm_runtime_addr_app_to_native(self.get_instance_pointer(), address as u64);

            if pointer.is_null() {
                return None;
            }

            if (pointer as usize) % core::mem::align_of::<T>() != 0 {
                return None;
            }

            Some(pointer as *mut T)
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn translate_to_guest<T>(&self, pointer: *mut T) -> WasmPointer {
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
        &mut self,
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
                &mut self.0,
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
    pub fn create_environment<'a>(&self, stack_size: usize) -> Result<&'a Self> {
        let execution_environment =
            unsafe { wasm_runtime_create_exec_env(self.get_instance_pointer(), stack_size as u32) };

        if execution_environment.is_null() {
            return Err(Error::ExecutionError(
                "Execution environment creation failed".to_string(),
            ));
        }

        Ok(unsafe { Self::from_raw_pointer(execution_environment) })
    }

    fn get_instance_pointer(&mut self) -> wasm_module_inst_t {
        unsafe { wasm_runtime_get_module_inst(&mut self.0) }
    }
}
