use core::{
    ffi::c_void,
    ops::{Deref, DerefMut},
};

use crate::host::virtual_machine::{InstanceReference, Result, module::Module, runtime::Runtime};
use wamr_rust_sdk::{instance, sys::wasm_runtime_set_custom_data};
use wasm_abi_bindings::InstanceContext;

pub struct Instance<'module>(instance::Instance<'module>);

unsafe impl Send for Instance<'_> {}

impl<'module> Instance<'module> {
    pub fn new(
        runtime: &Runtime,
        module: &'module Module<'module>,
        context: &'module mut InstanceContext,
        stack_size: usize,
    ) -> Result<Self> {
        let sdk_instance = instance::Instance::new(
            runtime.get_inner_reference(),
            module.get_inner_reference(),
            stack_size as u32,
        )?;

        unsafe {
            // Assuming the SDK provides a way to get the raw C pointer:
            let raw_ptr = sdk_instance.get_inner_instance();
            wasm_runtime_set_custom_data(raw_ptr, context as *mut InstanceContext as *mut c_void);
        }

        Ok(Self(sdk_instance))
    }
}

impl<'module> AsRef<instance::Instance<'module>> for Instance<'module> {
    fn as_ref(&self) -> &instance::Instance<'module> {
        &self.0
    }
}

impl<'module> AsMut<instance::Instance<'module>> for Instance<'module> {
    fn as_mut(&mut self) -> &mut instance::Instance<'module> {
        &mut self.0
    }
}

// Magic: Make the Owned type act like the Borrowed type!
impl<'module> Deref for Instance<'module> {
    type Target = InstanceReference;

    fn deref(&self) -> &Self::Target {
        unsafe { InstanceReference::from_raw_pointer(self.0.get_inner_instance()) }
    }
}

impl<'module> DerefMut for Instance<'module> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { InstanceReference::from_raw_pointer_mut(self.0.get_inner_instance()) }
    }
}
