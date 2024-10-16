#![allow(non_camel_case_types)]

use std::{ffi::CStr, marker::PhantomData, mem::size_of, os::raw::c_void, ptr::null_mut};

use wamr_rust_sdk::{
    sys::{
        wasm_exec_env_t, wasm_module_inst_t, wasm_runtime_addr_app_to_native,
        wasm_runtime_addr_native_to_app, wasm_runtime_call_indirect, wasm_runtime_create_exec_env,
        wasm_runtime_get_exception, wasm_runtime_get_exec_env_singleton,
        wasm_runtime_get_module_inst, wasm_runtime_get_user_data, wasm_runtime_module_free,
        wasm_runtime_module_malloc, wasm_runtime_set_user_data, wasm_runtime_validate_app_addr,
        wasm_runtime_validate_native_addr,
    },
    value::WasmValue,
};

use crate::{
    Data::Data_type, Error_type, Instance_type, Result_type, WASM_pointer_type, WASM_usize_type,
};

pub type Environment_pointer_type = wasm_exec_env_t;

#[derive(Debug, Clone, Copy)]
pub struct Environment_type<'a>(Environment_pointer_type, PhantomData<&'a ()>);

unsafe impl Send for Environment_type<'_> {}

unsafe impl Sync for Environment_type<'_> {}

impl<'a> Environment_type<'a> {
    pub fn From_raw_pointer(Raw_pointer: Environment_pointer_type) -> Result_type<Self> {
        if Raw_pointer.is_null() {
            return Err(Error_type::Invalid_pointer);
        }

        Ok(Self(Raw_pointer as Environment_pointer_type, PhantomData))
    }

    pub fn From_instance(Instance: &Instance_type) -> Result_type<Self> {
        let Instance_pointer = Instance.Get_inner_reference().get_inner_instance();

        if Instance_pointer.is_null() {
            return Err(Error_type::Invalid_pointer);
        }
        Ok(Self(
            unsafe { wasm_runtime_get_exec_env_singleton(Instance_pointer) },
            PhantomData,
        ))
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the user data is valid pointer.
    pub(crate) fn Set_user_data(&mut self, User_data: &Data_type) {
        unsafe {
            wasm_runtime_set_user_data(
                self.0,
                User_data as *const Data_type as *mut std::ffi::c_void,
            );
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the user data is valid pointer.
    pub fn Get_user_data(&self) -> &Data_type {
        unsafe {
            let User_data = wasm_runtime_get_user_data(self.0);

            if User_data.is_null() {
                panic!("Virtual machine user data is null");
            }
            &*(User_data as *const Data_type)
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn Convert_to_native_pointer<T>(&self, Address: WASM_pointer_type) -> *mut T {
        wasm_runtime_addr_app_to_native(self.Get_instance_pointer(), Address as u64) as *mut T
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
    pub unsafe fn Convert_to_WASM_pointer<T>(&self, Pointer: *const T) -> WASM_pointer_type {
        wasm_runtime_addr_native_to_app(self.Get_instance_pointer(), Pointer as *mut c_void)
            as WASM_pointer_type
    }

    pub fn Validate_WASM_pointer(&self, Address: WASM_pointer_type, Size: WASM_usize_type) -> bool {
        unsafe {
            wasm_runtime_validate_app_addr(self.Get_instance_pointer(), Address as u64, Size as u64)
        }
    }

    pub fn Validate_native_pointer<T>(&self, Pointer: *const T, Size: u64) -> bool {
        unsafe {
            wasm_runtime_validate_native_addr(
                self.Get_instance_pointer(),
                Pointer as *mut c_void,
                Size,
            )
        }
    }

    pub fn Convert_to_native_string(
        &self,
        Address: WASM_pointer_type,
        Size: WASM_usize_type,
    ) -> Result_type<&str> {
        if !self.Validate_WASM_pointer(Address, Size) {
            return Err(Error_type::Invalid_pointer);
        }
        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Str = unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(Pointer as *const u8, Size))
                .map_err(|_| Error_type::Invalid_UTF8_string)?
        };

        Ok(Str)
    }

    pub fn Convert_to_native_slice<T>(
        &self,
        Address: WASM_pointer_type,
        Length: WASM_usize_type,
    ) -> Result_type<&[T]> {
        if !self.Validate_WASM_pointer(Address, Length) {
            return Err(Error_type::Invalid_pointer);
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Slice = unsafe { std::slice::from_raw_parts(Pointer as *const T, Length) };

        Ok(Slice)
    }

    pub fn Convert_to_native_mutable_slice<T>(
        &self,
        Slice: WASM_pointer_type,
        Size: WASM_usize_type,
    ) -> Result_type<&'a mut [T]> {
        if !self.Validate_WASM_pointer(Slice, Size) {
            return Err(Error_type::Invalid_pointer);
        }

        let Slice_pointer = unsafe { self.Convert_to_native_pointer(Slice) };

        let Slice = unsafe { std::slice::from_raw_parts_mut(Slice_pointer, Size) };

        Ok(Slice)
    }

    pub fn Convert_to_native_reference<T>(&self, Address: WASM_pointer_type) -> Result_type<&'a T> {
        if !self.Validate_WASM_pointer(Address, size_of::<T>() as WASM_usize_type) {
            return Err(Error_type::Invalid_pointer);
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Reference = unsafe { &*(Pointer as *const T) };

        Ok(Reference)
    }

    pub fn Convert_to_native_mutable_reference<T>(
        &self,
        Address: WASM_pointer_type,
    ) -> Result_type<&'a mut T> {
        if !self.Validate_WASM_pointer(Address, size_of::<T>() as WASM_usize_type) {
            return Err(Error_type::Invalid_pointer);
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Reference = unsafe { &mut *(Pointer) };

        Ok(Reference)
    }

    pub fn Allocate<T>(&self, Size: WASM_usize_type) -> Result_type<&mut [T]> {
        let mut Pointer: *mut c_void = null_mut();

        unsafe {
            wasm_runtime_module_malloc(
                self.Get_instance_pointer(),
                Size as u64 * size_of::<T>() as u64,
                &mut Pointer as *mut *mut _,
            );
        }

        if Pointer.is_null() {
            return Err(Error_type::Allocation_failure);
        }

        if !self.Validate_native_pointer(Pointer as *const T, Size as u64 * size_of::<T>() as u64) {
            return Err(Error_type::Invalid_pointer);
        }

        Ok(unsafe { std::slice::from_raw_parts_mut(Pointer as *mut T, Size) })
    }

    pub fn Deallocate<T>(&self, Slice: &mut [T]) {
        let Pointer = unsafe { self.Convert_to_WASM_pointer(Slice.as_ptr()) };

        unsafe {
            wasm_runtime_module_free(self.Get_instance_pointer(), Pointer as u64);
        }
    }

    /// Make an indirect function call (call a function by its index which is not exported).
    /// For exported functions use `Call_export_function`.
    pub fn Call_indirect_function(
        &self,
        Function_index: u32,
        Parameters: &Vec<WasmValue>,
    ) -> Result_type<()> {
        let mut Arguments = Vec::new();

        for Parameter in Parameters {
            Arguments.append(&mut Parameter.encode());
        }

        if Arguments.is_empty() {
            Arguments.append(&mut WasmValue::I32(0).encode());
        }

        if !unsafe {
            wasm_runtime_call_indirect(
                self.0,
                Function_index,
                Arguments.len() as u32,
                Arguments.as_mut_ptr(),
            )
        } {
            let Exception_message =
                unsafe { wasm_runtime_get_exception(self.Get_instance_pointer()) };
            let Exception_message = unsafe { CStr::from_ptr(Exception_message) };
            let Exception_message =
                String::from_utf8_lossy(Exception_message.to_bytes()).to_string();

            return Err(Error_type::Execution_error(Exception_message));
        }

        Ok(())
    }

    /// Create a new execution environment.
    /// This environment should be initialized with `Initialize_thread_environment` and deinitialized with `Deinitialize_thread_environment`.
    pub fn Create_environment(&self, Stack_size: usize) -> Result_type<Self> {
        let Execution_environment =
            unsafe { wasm_runtime_create_exec_env(self.Get_instance_pointer(), Stack_size as u32) };

        if Execution_environment.is_null() {
            return Err(Error_type::Execution_error(
                "Execution environment creation failed".to_string(),
            ));
        }

        Ok(Self(Execution_environment, PhantomData))
    }

    fn Get_instance_pointer(&self) -> wasm_module_inst_t {
        unsafe { wasm_runtime_get_module_inst(self.0) }
    }

    #[allow(dead_code)]
    pub(crate) fn Get_inner_reference(&self) -> Environment_pointer_type {
        self.0
    }
}
