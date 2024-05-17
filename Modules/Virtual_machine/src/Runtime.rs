#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use wamr_rust_sdk::{
    runtime::{Runtime, RuntimeBuilder},
    RuntimeError,
};
use ABI::ABI_trait;

pub struct Runtime_builder_type(RuntimeBuilder);

impl Runtime_builder_type {
    pub fn New() -> Self {
        let Runtime_builder = Runtime::builder().use_system_allocator();

        Self(Runtime_builder)
    }

    pub fn Register_function(self, Name: &str, Function_pointer: *mut std::ffi::c_void) -> Self {
        Self(self.0.register_host_function(Name, Function_pointer))
    }

    pub fn Register_ABI(self, ABI: impl ABI_trait) -> Self {
        let mut Runtime_builder = self;

        for Function in ABI.Get_functions() {
            Runtime_builder =
                Runtime_builder.Register_function(Function.Name, Function.Function_pointer);
        }

        Runtime_builder
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
}
