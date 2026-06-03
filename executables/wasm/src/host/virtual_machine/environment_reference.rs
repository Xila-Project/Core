use wamr_rust_sdk::sys::{
    WASMExecEnv, wasm_runtime_get_module_inst, wasm_runtime_get_user_data,
    wasm_runtime_set_instruction_count_limit,
};
use wasm_abi_bindings::EnvironmentContext;

use crate::host::virtual_machine::InstanceReference;

#[repr(transparent)]
#[derive(Debug)]
pub struct EnvironmentReference(WASMExecEnv);

impl EnvironmentReference {
    /// Safe wrapper to get the C pointer out if needed for WAMR C API calls
    pub fn as_raw_pointer(&self) -> *mut WASMExecEnv {
        // Because of repr(transparent), the address of self IS the raw pointer
        self as *const Self as *mut Self as *mut WASMExecEnv
    }

    /// Turn a raw C pointer into a shared reference
    pub unsafe fn from_raw_pointer<'a>(raw_pointer: *const WASMExecEnv) -> &'a Self {
        unsafe { &*(raw_pointer as *const Self) }
    }

    /// Turn a raw C pointer into a mutable reference
    pub unsafe fn from_raw_pointer_mut<'a>(raw_pointer: *mut WASMExecEnv) -> &'a mut Self {
        unsafe { &mut *(raw_pointer as *mut Self) }
    }

    pub fn get_context(&mut self) -> &mut EnvironmentContext {
        unsafe {
            let context = wasm_runtime_get_user_data(self.as_raw_pointer());
            &mut *(context as *mut EnvironmentContext)
        }
    }

    pub fn set_instruction_count_limit(&mut self, limit: Option<u32>) {
        let limit = limit.map(|l| l as i32).unwrap_or(-1);
        unsafe {
            wasm_runtime_set_instruction_count_limit(self.as_raw_pointer(), limit);
        }
    }

    pub fn get_instance(&self) -> &mut InstanceReference {
        unsafe {
            let instance_ptr = wasm_runtime_get_module_inst(self.as_raw_pointer());
            InstanceReference::from_raw_pointer_mut(instance_ptr)
        }
    }
}
