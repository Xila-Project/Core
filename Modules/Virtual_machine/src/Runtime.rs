use wamr_rust_sdk::{
    runtime::{Runtime, RuntimeBuilder},
    sys::{wasm_runtime_destroy_thread_env, wasm_runtime_init_thread_env},
};

use crate::{Registrable_trait, Result_type};

pub struct Runtime_builder_type(RuntimeBuilder);

impl Runtime_builder_type {
    pub fn New() -> Self {
        let Runtime_builder = Runtime::builder().use_system_allocator();

        Self(Runtime_builder)
    }

    pub fn Register_function(
        mut self,
        Name: &str,
        Function_pointer: *mut std::ffi::c_void,
    ) -> Self {
        self.0 = self.0.register_host_function(Name, Function_pointer);
        self
    }

    pub fn Register(mut self, Registrable: impl Registrable_trait) -> Self {
        for Function_descriptor in Registrable.Get_functions() {
            self = self.Register_function(Function_descriptor.Name, Function_descriptor.Pointer);
        }

        self
    }

    pub fn Build(self) -> Result<Runtime_type, RuntimeError> {
        Ok(Runtime_type(self.0.build()?))
    }
}

pub struct Runtime_type(Runtime);

impl Runtime_type {
    pub fn Builder() -> Runtime_builder_type {
        Runtime_builder_type::New()
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
