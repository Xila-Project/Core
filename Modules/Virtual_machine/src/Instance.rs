#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use wamr_rust_sdk::{function::Function, instance::Instance, value::WasmValue, RuntimeError};

use crate::{Module::Module_type, Runtime::Runtime_type};

pub struct Instance_type(Instance);

impl Instance_type {
    pub fn New(
        Runtime: &Runtime_type,
        Module: &Module_type,
        Stack_size: usize,
    ) -> Result<Self, RuntimeError> {
        Ok(Instance_type(Instance::new(
            Runtime.Get_inner_reference(),
            Module.Get_inner_reference(),
            Stack_size as u32,
        )?))
    }

    pub fn Call_export_function(
        &self,
        Name: &str,
        Parameters: &Vec<WasmValue>,
    ) -> Result<WasmValue, RuntimeError> {
        Function::find_export_func(self.Get_inner_reference(), Name)?.call(&self.0, Parameters)
    }

    pub fn Call_main(&self, Parameters: &Vec<WasmValue>) -> Result<WasmValue, RuntimeError> {
        Function::find_export_func(self.Get_inner_reference(), "main")?.call(&self.0, Parameters)
    }

    pub(crate) fn Get_inner_reference(&self) -> &Instance {
        &self.0
    }
}
