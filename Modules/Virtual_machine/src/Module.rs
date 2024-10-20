use wamr_rust_sdk::{module::Module, RuntimeError};

use crate::Runtime::Runtime_type;

pub struct Module_type(Module);

impl Module_type {
    pub fn From_buffer(
        Runtime: &Runtime_type,
        Buffer: Vec<u8>,
        Name: &str,
    ) -> Result<Self, RuntimeError> {
        Ok(Module_type(Module::from_vec(
            Runtime.Get_inner_reference(),
            Buffer,
            Name,
        )?))
    }

    pub(crate) fn Get_inner_reference(&self) -> &Module {
        &self.0
    }
}
