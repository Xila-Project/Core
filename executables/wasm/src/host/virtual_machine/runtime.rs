//! WASM Runtime management and configuration.
//!
//! This module provides a wrapper around the WAMR runtime with a builder pattern
//! for configuring and creating runtime instances with registered host functions.

use core::ffi::c_void;

use crate::host::virtual_machine::{Registrable, Result};
use wamr_rust_sdk::{
    runtime,
    sys::{wasm_runtime_destroy_thread_env, wasm_runtime_init_thread_env},
};
use xila::synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

/// Builder for configuring and creating WASM runtime instances.
///
/// This builder allows incremental configuration of the runtime with
/// host functions before creating the final runtime instance.
pub struct RuntimeBuilder(runtime::RuntimeBuilder);

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        let runtime_builder = runtime::Runtime::builder().use_system_allocator();

        Self(runtime_builder)
    }

    pub fn register_function(self, name: &str, function_pointer: *mut c_void) -> Self {
        Self(self.0.register_host_function(name, function_pointer))
    }

    pub fn register(mut self, registrable: &dyn Registrable) -> Self {
        for function_descriptor in registrable.get_functions() {
            self = self.register_function(function_descriptor.name, function_descriptor.pointer);
        }

        self
    }

    pub fn build(self) -> Result<Runtime> {
        Ok(Runtime(self.0.build()?))
    }
}

pub struct Runtime(Mutex<CriticalSectionRawMutex, runtime::Runtime>);

unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

impl Runtime {
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    pub(crate) fn get_inner_reference(&self) -> &runtime::Runtime {
        &self.0
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
