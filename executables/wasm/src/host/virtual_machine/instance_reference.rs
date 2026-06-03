use core::ffi::c_void;
use wamr_rust_sdk::sys::{
    WASMModuleInstanceCommon, wasm_runtime_addr_app_to_native, wasm_runtime_addr_native_to_app,
    wasm_runtime_clear_exception, wasm_runtime_get_custom_data, wasm_runtime_get_exception,
};
use wasm_abi_bindings::{InstanceContext, WasmPointer};

#[repr(transparent)]
pub struct InstanceReference(WASMModuleInstanceCommon);

impl InstanceReference {
    pub fn as_raw_pointer(&self) -> wamr_rust_sdk::sys::wasm_module_inst_t {
        self as *const Self as *mut Self as wamr_rust_sdk::sys::wasm_module_inst_t
    }

    pub unsafe fn from_raw_pointer<'a>(ptr: wamr_rust_sdk::sys::wasm_module_inst_t) -> &'a Self {
        unsafe { &*(ptr as *const Self) }
    }

    pub unsafe fn from_raw_pointer_mut<'a>(
        ptr: wamr_rust_sdk::sys::wasm_module_inst_t,
    ) -> &'a mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }

    pub unsafe fn translate_to_host<T>(&mut self, address: WasmPointer) -> Option<*mut T> {
        unsafe {
            let pointer = wasm_runtime_addr_app_to_native(self.as_raw_pointer(), address as u64);
            if pointer.is_null() || !(pointer as usize).is_multiple_of(core::mem::align_of::<T>()) {
                return None;
            }
            Some(pointer as *mut T)
        }
    }

    pub unsafe fn translate_to_guest<T>(&mut self, pointer: *mut T) -> WasmPointer {
        unsafe {
            wasm_runtime_addr_native_to_app(self.as_raw_pointer(), pointer as *mut c_void)
                as WasmPointer
        }
    }

    pub fn get_context(&mut self) -> &mut InstanceContext {
        unsafe {
            let context = wasm_runtime_get_custom_data(self.as_raw_pointer());
            &mut *(context as *mut InstanceContext)
        }
    }

    pub fn get_current_exception(&self) -> Option<&str> {
        unsafe {
            let exception_ptr = wasm_runtime_get_exception(self.as_raw_pointer());
            if exception_ptr.is_null() {
                return None;
            }
            let exception_cstr = core::ffi::CStr::from_ptr(exception_ptr);
            let s = exception_cstr.to_str().unwrap_or("");
            if s.is_empty() { None } else { Some(s) }
        }
    }

    pub fn clear_exception(&self) {
        unsafe {
            wasm_runtime_clear_exception(self.as_raw_pointer());
        }
    }
}
