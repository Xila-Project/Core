use core::{
    ffi::c_void,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use wamr_rust_sdk::sys::{
    WASMExecEnv, wasm_runtime_create_exec_env, wasm_runtime_destroy_exec_env,
    wasm_runtime_set_user_data,
};
use wasm_abi_bindings::EnvironmentContext;
use xila::log;

use crate::host::virtual_machine::{EnvironmentReference, Error, Instance, Result};

// (Assuming Instance, EnvironmentContext, Error, and Result are imported)

#[derive(Debug)]
pub struct Environment<'instance> {
    raw_pointer: *mut WASMExecEnv,
    // We keep PhantomData to ensure this struct cannot outlive the context/instance
    _phantom: PhantomData<&'instance mut ()>,
}

impl<'instance> Environment<'instance> {
    pub fn new(
        instance: &Instance<'instance>,
        context: &'instance mut EnvironmentContext,
        stack_size: usize,
    ) -> Result<Self> {
        let raw_pointer =
            unsafe { wasm_runtime_create_exec_env(instance.as_raw_pointer(), stack_size as u32) };

        if raw_pointer.is_null() {
            return Err(Error::Execution(
                "failed to create WASM execution environment".into(),
            ));
        }

        unsafe {
            wasm_runtime_set_user_data(
                raw_pointer,
                context as *mut EnvironmentContext as *mut c_void,
            );
        }

        Ok(Self {
            raw_pointer,
            _phantom: PhantomData,
        })
    }
}

impl<'instance> Drop for Environment<'instance> {
    fn drop(&mut self) {
        unsafe {
            log::information!(
                "Dropping WASM execution environment: {:?}",
                self.raw_pointer
            );
            wasm_runtime_destroy_exec_env(self.raw_pointer);
        }
    }
}

// Magic happens here: The Owner transparently acts like the Reference!
impl Deref for Environment<'_> {
    type Target = EnvironmentReference;

    fn deref(&self) -> &Self::Target {
        unsafe { EnvironmentReference::from_raw_pointer(self.raw_pointer) }
    }
}

impl DerefMut for Environment<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { EnvironmentReference::from_raw_pointer_mut(self.raw_pointer) }
    }
}
