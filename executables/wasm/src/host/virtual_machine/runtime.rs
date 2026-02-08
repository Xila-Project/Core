//! WASM Runtime management and configuration.
//!
//! This module provides a wrapper around the WAMR runtime with a builder pattern
//! for configuring and creating runtime instances with registered host functions.

use crate::host::virtual_machine::{Error, Instance, Module, Registrable, Result};
use alloc::vec::Vec;
use wamr_rust_sdk::value::WasmValue;
use xila::{
    abi_context::{self, FileIdentifier},
    task::TaskIdentifier,
    virtual_file_system::File,
};

pub struct Runtime(wamr_rust_sdk::runtime::Runtime);

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

impl Runtime {
    pub fn new<'a>(registrables: impl IntoIterator<Item = &'a dyn Registrable>) -> Result<Self> {
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
        mut function_arguments: Vec<WasmValue>,
        task: TaskIdentifier,
    ) -> Result<Vec<WasmValue>> {
        let abi_context = abi_context::get_instance();

        if function_arguments.is_empty() {
            function_arguments.push(WasmValue::I32(0));
        }

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
}
