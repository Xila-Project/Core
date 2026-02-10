use crate::host::virtual_machine::{CustomData, Result, module::Module, runtime::Runtime};
use alloc::{boxed::Box, vec, vec::Vec};
use wamr_rust_sdk::{
    function::Function, instance, sys::wasm_runtime_get_custom_data, value::WasmValue,
};

pub struct Instance<'module> {
    instance: instance::Instance<'module>,
}

unsafe impl Send for Instance<'_> {}

impl Drop for Instance<'_> {
    fn drop(&mut self) {
        let instance = self.get_inner_reference().get_inner_instance();
        unsafe {
            let user_data = wasm_runtime_get_custom_data(instance) as *mut CustomData;

            if !user_data.is_null() {
                let _ = Box::from_raw(user_data);
            }
        }

        // User data is dropped here.
    }
}

impl<'module> Instance<'module> {
    pub fn new(
        runtime: &Runtime,
        module: &'module Module<'module>,
        stack_size: usize,
    ) -> Result<Self> {
        let wamr_instance = instance::Instance::new(
            runtime.get_inner_reference(),
            module.get_inner_reference(),
            stack_size as u32,
        )?;

        let instance = Instance {
            instance: wamr_instance,
        };

        Ok(instance)
    }

    pub fn call_exported_function(
        &self,
        name: &str,
        parameters: &Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>> {
        if parameters.is_empty() {
            Ok(
                Function::find_export_func(self.get_inner_reference(), name)?
                    .call(&self.instance, &vec![WasmValue::I32(0)])?,
            )
        } else {
            Ok(
                Function::find_export_func(self.get_inner_reference(), name)?
                    .call(&self.instance, parameters)?,
            )
        }
    }

    pub fn get_inner_reference(&'_ self) -> &'_ instance::Instance<'_> {
        &self.instance
    }
}
