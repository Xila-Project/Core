//! WASM Runtime management and configuration.
//!
//! This module provides a wrapper around the WAMR runtime with a builder pattern
//! for configuring and creating runtime instances with registered host functions.

use core::{
    ffi::{CStr, c_void},
    mem::forget,
};

use crate::host::virtual_machine::{Error, Instance, Module, Registrable, Result};
use alloc::{string::ToString, vec::Vec};
use wamr_rust_sdk::{
    runtime,
    sys::{
        wasm_runtime_destroy_thread_env, wasm_runtime_init_thread_env, wasm_runtime_is_xip_file,
        wasm_runtime_load, wasm_runtime_register_module,
    },
    value::WasmValue,
};
use xila::{
    abi_context::{self, FileIdentifier},
    synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex},
    task::TaskIdentifier,
    virtual_file_system::File,
};

pub struct Runtime(wamr_rust_sdk::runtime::Runtime);

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

impl Runtime {
    pub fn new(registrables: &[&dyn Registrable]) -> Result<Self> {
        let mut runtime_builder = wamr_rust_sdk::runtime::Runtime::builder().use_system_allocator();

        for registrable in registrables {
            for function_descriptor in registrable.get_functions() {
                runtime_builder = runtime_builder
                    .register_host_function(function_descriptor.name, function_descriptor.pointer);
            }
        }

        let runtime = runtime_builder.build()?;

        Ok(Self(runtime))
    }

    pub fn get_inner_reference(&self) -> &wamr_rust_sdk::runtime::Runtime {
        &self.0
    }

    /// Load a WASM module from a buffer for execution.
    ///
    /// This method loads a WASM module into the runtime, either as a regular module
    /// or as an XIP (execute-in-place) module for AOT compiled binaries.
    ///
    /// # Arguments
    ///
    /// * `Buffer` - The WASM module bytecode
    /// * `XIP` - Whether this is an XIP AOT compiled module
    /// * `Name` - Name to register the module under
    ///
    /// # Returns
    ///
    /// Success or an error if loading fails
    ///
    /// # Errors
    ///
    /// Returns an error if the module is not an XIP AOT compiled module or if the module cannot be loaded from the buffer.
    fn load_module(&self, buffer: &[u8], xip: bool, name: &str) -> Result<()> {
        if unsafe { xip && !wasm_runtime_is_xip_file(buffer.as_ptr(), buffer.len() as u32) } {
            return Err(Error::InvalidModule);
        }

        unsafe {
            let mut buffer = if xip {
                Vec::from_raw_parts(buffer.as_ptr() as *mut u8, buffer.len(), buffer.len())
            } else {
                buffer.to_vec()
            };

            let mut error_buffer = [0_i8; 128];

            let module = wasm_runtime_load(
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                error_buffer.as_mut_ptr(),
                error_buffer.len() as u32,
            );

            if module.is_null() {
                return Err(Error::CompilationError(
                    CStr::from_ptr(error_buffer.as_ptr())
                        .to_string_lossy()
                        .to_string(),
                ));
            }

            if !wasm_runtime_register_module(
                name.as_ptr() as *const i8,
                module,
                error_buffer.as_mut_ptr(),
                error_buffer.len() as u32,
            ) {
                return Err(Error::InternalError);
            }

            forget(buffer);
        }

        Ok(())
    }

    /// Execute a WASM module with the specified I/O configuration.
    ///
    /// This is the main entry point for executing WASM modules. It creates a new
    /// module instance, sets up the execution environment with proper I/O redirection,
    /// and calls the module's main function.
    ///
    /// # Arguments
    ///
    /// * `Buffer` - The WASM module bytecode to execute
    /// * `Stack_size` - Stack size in bytes for the WASM instance
    /// * `Standard_in` - File identifier for standard input
    /// * `Standard_out` - File identifier for standard output  
    /// * `Standard_error` - File identifier for standard error
    ///
    /// # Returns
    ///
    /// The return values from the WASM module's main function
    ///
    /// # Errors
    ///
    /// Returns an error if module loading, instantiation, or execution fails
    pub async fn execute(
        &'static self,
        name: &str,
        buffer: Vec<u8>,
        stack_size: usize,
        (standard_in, standard_out, standard_error): (File, File, File),
        function_name: &str,
        function_arguments: Vec<WasmValue>,
        task: TaskIdentifier,
    ) -> Result<Vec<WasmValue>> {
        let abi_context = abi_context::get_instance();

        abi_context
            .call_abi(async || {
                let standard_in = abi_context
                    .insert_file(
                        task,
                        standard_in.into_synchronous_file(),
                        Some(FileIdentifier::STANDARD_IN),
                    )
                    .ok_or(Error::FailedToRegisterFileContext)?;
                let standard_out = abi_context
                    .insert_file(
                        task,
                        standard_out.into_synchronous_file(),
                        Some(FileIdentifier::STANDARD_OUT),
                    )
                    .ok_or(Error::FailedToRegisterFileContext)?;
                let standard_error = abi_context
                    .insert_file(
                        task,
                        standard_error.into_synchronous_file(),
                        Some(FileIdentifier::STANDARD_ERROR),
                    )
                    .ok_or(Error::FailedToRegisterFileContext)?;

                let module = Module::from_buffer(
                    &self,
                    buffer,
                    name,
                    standard_in,
                    standard_out,
                    standard_error,
                )
                .await?;

                let instance = Instance::new(&self, &module, stack_size as _)?;

                let result = instance.call_exported_function(function_name, &function_arguments)?;

                Ok(result)
            })
            .await
    }

    pub fn initialize_thread_environment() -> Option<()> {
        if unsafe { wasm_runtime_init_thread_env() } {
            Some(())
        } else {
            None
        }
    }

    pub fn deinitialize_thread_environment() {
        unsafe { wasm_runtime_destroy_thread_env() }
    }
}
