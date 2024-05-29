#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{marker::PhantomData, mem::size_of, os::raw::c_void, ptr::NonNull};

use wamr_rust_sdk::sys::{
    wasm_exec_env_t, wasm_module_inst_t, wasm_runtime_addr_app_to_native,
    wasm_runtime_addr_native_to_app, wasm_runtime_get_exec_env_singleton,
    wasm_runtime_get_module_inst, wasm_runtime_get_user_data, wasm_runtime_set_user_data,
    wasm_runtime_validate_app_addr, wasm_runtime_validate_native_addr,
};
use Shared::{Mutable_slice_type, Mutable_string_type};

use crate::{Instance_type, WASM_pointer, WASM_usize};

pub type Environment_pointer_type = wasm_exec_env_t;

pub struct Environment_type<'a>(Environment_pointer_type, PhantomData<&'a ()>);

impl<'a> Environment_type<'a> {
    pub fn From_raw_pointer(Raw_pointer: Environment_pointer_type) -> Result<Self, String> {
        if Raw_pointer.is_null() {
            return Err("Pointer is null".to_string());
        }

        Ok(Self(Raw_pointer as Environment_pointer_type, PhantomData))
    }

    pub fn From_instance(Instance: &Instance_type) -> Result<Self, String> {
        let Instance_pointer = Instance.Get_inner_reference().get_inner_instance();

        if Instance_pointer.is_null() {
            return Err("Instance pointer is null".to_string());
        }
        Ok(Self(
            unsafe { wasm_runtime_get_exec_env_singleton(Instance_pointer) },
            PhantomData,
        ))
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the user data is valid pointer.
    pub unsafe fn Set_user_data<T>(&mut self, User_data: &mut T) {
        wasm_runtime_set_user_data(self.0, User_data as *mut T as *mut std::ffi::c_void);
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the user data is valid pointer.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn Get_user_data<T>(&self) -> Option<&mut T> {
        let User_data = wasm_runtime_get_user_data(self.0);

        if User_data.is_null() {
            None
        } else {
            Some(&mut *(User_data as *mut T))
        }
    }

    pub unsafe fn Convert_to_native_pointer(&self, Address: WASM_pointer) -> *mut c_void {
        wasm_runtime_addr_app_to_native(self.Get_instance_pointer(), Address as u64) as *mut c_void
    }

    pub unsafe fn Convert_to_WASM_pointer<T>(&self, Pointer: *const T) -> WASM_pointer {
        wasm_runtime_addr_native_to_app(self.Get_instance_pointer(), Pointer as *mut c_void)
            as WASM_pointer
    }

    pub fn Validate_WASM_pointer(&self, Address: WASM_pointer, Size: WASM_usize) -> bool {
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
        Address: WASM_pointer,
        Size: WASM_usize,
    ) -> Result<&str, String> {
        if !self.Validate_WASM_pointer(Address, Size) {
            return Err("Invalid pointer".to_string());
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Str = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                Pointer as *const u8,
                Size as usize,
            ))
        };

        Ok(Str)
    }

    pub fn Convert_to_native_mutable_string(
        &self,
        Address: WASM_pointer,
        Length: WASM_pointer,
        Size: WASM_usize,
    ) -> Result<Mutable_string_type<'a>, String> {
        if !self.Validate_WASM_pointer(Address, Size) {
            return Err("Invalid pointer to string".to_string());
        }
        if !self.Validate_WASM_pointer(Length, size_of::<WASM_pointer>() as WASM_usize) {
            return Err("Invalid pointer to length".to_string());
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        Mutable_string_type::<'a>::From(
            unsafe { NonNull::new_unchecked(Pointer as *mut u8) },
            unsafe { NonNull::new_unchecked(Length as *mut WASM_usize) },
            Size,
        )
        .map_err(|_| "Invalid UTF-8 string".to_string())
    }

    pub fn Convert_to_native_slice<T>(
        &self,
        Address: WASM_pointer,
        Length: WASM_usize,
    ) -> Result<&[T], String> {
        if !self.Validate_WASM_pointer(Address, Length) {
            return Err("Invalid pointer".to_string());
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Slice = unsafe { std::slice::from_raw_parts(Pointer as *const T, Length as usize) };

        Ok(Slice)
    }

    pub fn Convert_to_native_mutable_slice<T>(
        &self,
        Slice: WASM_pointer,
        Length: WASM_pointer,
        Size: WASM_usize,
    ) -> Result<Mutable_slice_type<'a, T>, String> {
        if !self.Validate_WASM_pointer(Slice, Size) {
            return Err("Invalid pointer to slice".to_string());
        }
        if !self.Validate_WASM_pointer(Length, size_of::<WASM_usize>() as WASM_usize) {
            return Err("Invalid pointer to length".to_string());
        }

        let Slice_pointer = unsafe { self.Convert_to_native_pointer(Slice) };
        let Length_pointer = unsafe { self.Convert_to_native_pointer(Length) };

        Ok(Mutable_slice_type::<'a, T>::From(
            NonNull::new(Slice_pointer as *mut T).ok_or("Invalid pointer")?,
            NonNull::new(Length_pointer as *mut WASM_usize).ok_or("Invalid pointer")?,
            Size,
        )?)
    }

    fn Get_instance_pointer(&self) -> wasm_module_inst_t {
        unsafe { wasm_runtime_get_module_inst(self.0) }
    }

    #[allow(dead_code)]
    pub(crate) fn Get_inner_reference(&self) -> Environment_pointer_type {
        self.0
    }
}
