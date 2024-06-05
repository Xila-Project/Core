use std::{marker::PhantomData, mem::size_of, os::raw::c_void, ptr::NonNull};

use wamr_rust_sdk::sys::{
    wasm_exec_env_t, wasm_module_inst_t, wasm_runtime_addr_app_to_native,
    wasm_runtime_addr_native_to_app, wasm_runtime_get_exec_env_singleton,
    wasm_runtime_get_module_inst, wasm_runtime_get_user_data, wasm_runtime_set_user_data,
    wasm_runtime_validate_app_addr, wasm_runtime_validate_native_addr,
};
use Shared::Mutable_string_type;

use crate::{Data::Data_type, Error_type, Instance_type, WASM_pointer, WASM_usize};

pub type Environment_pointer_type = wasm_exec_env_t;

pub struct Environment_type<'a>(Environment_pointer_type, PhantomData<&'a ()>);

impl<'a> Environment_type<'a> {
    pub fn From_raw_pointer(Raw_pointer: Environment_pointer_type) -> Result<Self, Error_type> {
        if Raw_pointer.is_null() {
            return Err(Error_type::Invalid_pointer);
        }

        Ok(Self(Raw_pointer as Environment_pointer_type, PhantomData))
    }

    pub fn From_instance(Instance: &Instance_type) -> Result<Self, Error_type> {
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
    #[allow(clippy::mut_from_ref)]
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
    pub unsafe fn Convert_to_native_pointer(&self, Address: WASM_pointer) -> *mut c_void {
        wasm_runtime_addr_app_to_native(self.Get_instance_pointer(), Address as u64) as *mut c_void
    }

    /// # Safety
    ///
    /// This function is unsafe because it is not checked that the address is valid.
    /// Please use `Validate_WASM_pointer` to check the address.
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
    ) -> Result<&str, Error_type> {
        if !self.Validate_WASM_pointer(Address, Size) {
            return Err(Error_type::Invalid_pointer);
        }
        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Str = unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(
                Pointer as *const u8,
                Size as usize,
            ))
            .map_err(|_| Error_type::Invalid_UTF8_string)?
        };

        Ok(Str)
    }

    pub fn Convert_to_native_mutable_string(
        &self,
        String: WASM_pointer,
        Length: WASM_pointer,
        Size: WASM_usize,
    ) -> Result<Mutable_string_type<'a>, Error_type> {
        if !self.Validate_WASM_pointer(String, Size) {
            return Err(Error_type::Invalid_pointer);
        }
        if !self.Validate_WASM_pointer(Length, size_of::<WASM_pointer>() as WASM_usize) {
            return Err(Error_type::Invalid_pointer);
        }
        let String_pointer = unsafe { self.Convert_to_native_pointer(String) };
        let Length_pointer = unsafe { self.Convert_to_native_pointer(Length) };

        Mutable_string_type::<'a>::From(
            unsafe { NonNull::new_unchecked(String_pointer as *mut u8) },
            unsafe { NonNull::new_unchecked(Length_pointer as *mut WASM_usize) },
            Size,
        )
        .map_err(Error_type::Slice_conversion_failed)
    }

    pub fn Convert_to_native_slice<T>(
        &self,
        Address: WASM_pointer,
        Length: WASM_usize,
    ) -> Result<&[T], Error_type> {
        if !self.Validate_WASM_pointer(Address, Length) {
            return Err(Error_type::Invalid_pointer);
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Slice = unsafe { std::slice::from_raw_parts(Pointer as *const T, Length as usize) };

        Ok(Slice)
    }

    pub fn Convert_to_native_mutable_slice<T>(
        &self,
        Slice: WASM_pointer,
        Size: WASM_usize,
    ) -> Result<&'a mut [T], Error_type> {
        if !self.Validate_WASM_pointer(Slice, Size) {
            return Err(Error_type::Invalid_pointer);
        }

        let Slice_pointer = unsafe { self.Convert_to_native_pointer(Slice) };

        let Slice =
            unsafe { std::slice::from_raw_parts_mut(Slice_pointer as *mut T, Size as usize) };

        Ok(Slice)
    }

    pub fn Convert_to_native_reference<T>(
        &self,
        Address: WASM_pointer,
    ) -> Result<&'a T, Error_type> {
        if !self.Validate_WASM_pointer(Address, size_of::<T>() as WASM_usize) {
            return Err(Error_type::Invalid_pointer);
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Reference = unsafe { &*(Pointer as *const T) };

        Ok(Reference)
    }

    pub fn Convert_to_native_mutable_reference<T>(
        &self,
        Address: WASM_pointer,
    ) -> Result<&'a mut T, Error_type> {
        if !self.Validate_WASM_pointer(Address, size_of::<T>() as WASM_usize) {
            return Err(Error_type::Invalid_pointer);
        }

        let Pointer = unsafe { self.Convert_to_native_pointer(Address) };

        let Reference = unsafe { &mut *(Pointer as *mut T) };

        Ok(Reference)
    }

    fn Get_instance_pointer(&self) -> wasm_module_inst_t {
        unsafe { wasm_runtime_get_module_inst(self.0) }
    }

    #[allow(dead_code)]
    pub(crate) fn Get_inner_reference(&self) -> Environment_pointer_type {
        self.0
    }
}
