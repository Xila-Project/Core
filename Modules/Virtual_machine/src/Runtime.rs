//! WASM Runtime management and configuration.
//!
//! This module provides a wrapper around the WAMR runtime with a builder pattern
//! for configuring and creating runtime instances with registered host functions.

use core::ffi::c_void;

use wamr_rust_sdk::{
    runtime::{Runtime, RuntimeBuilder},
    sys::{wasm_runtime_destroy_thread_env, wasm_runtime_init_thread_env},
};

use crate::{Registrable_trait, Result_type};

/// Builder for configuring and creating WASM runtime instances.
///
/// This builder allows incremental configuration of the runtime with
/// host functions before creating the final runtime instance.
pub struct Runtime_builder_type(RuntimeBuilder);

impl Runtime_builder_type {
    pub fn new() -> Self {
        let runtime_builder = Runtime::builder().use_system_allocator();

        Self(runtime_builder)
    }

    pub fn Register_function(self, Name: &str, Function_pointer: *mut c_void) -> Self {
        Self(self.0.register_host_function(Name, Function_pointer))
    }

    pub fn Register(mut self, Registrable: &dyn Registrable_trait) -> Self {
        for function_descriptor in Registrable.get_functions() {
            self = self.Register_function(function_descriptor.name, function_descriptor.pointer);
        }

        self
    }

    pub fn Build(self) -> Result_type<Runtime_type> {
        Ok(Runtime_type(self.0.build()?))
    }
}

pub struct Runtime_type(Runtime);

impl Runtime_type {
    pub fn builder() -> Runtime_builder_type {
        Runtime_builder_type::new()
    }

    pub(crate) fn Get_inner_reference(&self) -> &Runtime {
        &self.0
    }

    pub fn Initialize_thread_environment() -> Option<()> {
        if unsafe { wasm_runtime_init_thread_env() } {
            Some(())
        } else {
            None
        }
    }

    pub fn Deinitialize_thread_environment() {
        unsafe { wasm_runtime_destroy_thread_env() }
    }
}
