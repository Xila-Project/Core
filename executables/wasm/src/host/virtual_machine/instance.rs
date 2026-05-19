use crate::host::virtual_machine::{Result, module::Module, runtime::Runtime};
use alloc::boxed::Box;
use wamr_rust_sdk::{instance, sys::wasm_runtime_get_custom_data};
use wasm_abi_bindings::InstanceContext;

pub struct Instance<'module> {
    instance: instance::Instance<'module>,
}

unsafe impl Send for Instance<'_> {}

impl Drop for Instance<'_> {
    fn drop(&mut self) {
        let instance = self.get_inner_reference().get_inner_instance();
        unsafe {
            let user_data = wasm_runtime_get_custom_data(instance) as *mut InstanceContext;

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

    pub fn get_inner_reference(&'_ self) -> &'_ instance::Instance<'_> {
        &self.instance
    }
}
