use std::{
    ffi::CStr,
    ffi::{c_char, CString},
    ptr::null_mut,
};
use wamr_sys::*;

use super::{Function::Function_type, Symbol::Symbol_type};

pub struct Module_builder_type {
    Name: CString,
    Executable: Vec<u8>,
    Module: wasm_module_t,
    Error_buffer: Vec<i8>,
    Stack_size: usize,
    Heap_size: usize,
    Arguments: Vec<CString>,
}

const Default_stack_size: usize = 8 * 1024;
const Default_host_managed_heap_size: usize = 8 * 1024;
const Default_error_buffer_size: usize = 128;

impl Module_builder_type {
    pub fn Load(
        Name: &str,
        mut Executable: Vec<u8>,
        Error_buffer_size: Option<usize>,
    ) -> Result<Self, ()> {
        let Name = std::ffi::CString::new(Name).unwrap();
        let mut Error_buffer: Vec<i8> =
            vec![0; Error_buffer_size.unwrap_or(Default_error_buffer_size)];

        let Module = unsafe {
            wasm_runtime_load(
                Executable.as_mut_ptr(),
                Executable.len() as u32,
                Error_buffer.as_mut_ptr(),
                Error_buffer.len() as u32,
            )
        };

        if Module.is_null() {
            return Err(());
        }
        Ok(Self {
            Name,
            Executable: Executable, // ! : Not sure, check if no copy is made
            Module,
            Error_buffer,
            Stack_size: Default_stack_size,
            Heap_size: Default_host_managed_heap_size,
            Arguments: vec![],
        })
    }

    pub fn Set_stack_size(mut self, Stack_size: usize) -> Self {
        self.Stack_size = Stack_size;
        self
    }

    pub fn Set_host_managed_heap_size(mut self, Host_managed_heap_size: usize) -> Self {
        self.Heap_size = Host_managed_heap_size;
        self
    }

    pub fn Set_arguments(mut self, Arguments: Vec<&str>) -> Self {
        self.Arguments = Arguments
            .iter()
            .map(|Argument| CString::new(*Argument).unwrap())
            .collect();

        unsafe {
            wasm_runtime_set_wasi_args(
                self.Module,
                null_mut(),
                0,
                null_mut(),
                0,
                null_mut(),
                0,
                self.Arguments.as_mut_ptr() as *mut *mut i8,
                self.Arguments.len() as i32,
            );
        }
        self
    }

    pub fn Build(mut self) -> Result<Module_instance_type, ()> {
        let Module_instance = unsafe {
            wasm_runtime_instantiate(
                self.Module,
                self.Stack_size as u32,
                self.Heap_size as u32,
                self.Error_buffer.as_mut_ptr(),
                self.Error_buffer.len() as u32,
            )
        };

        if Module_instance.is_null() {
            return Err(());
        }

        Ok(Module_instance_type {
            Builder: self,
            Module_instance,
        })
    }
}

pub struct Module_instance_type {
    Builder: Module_builder_type,
    Module_instance: wasm_module_inst_t,
}

impl Drop for Module_instance_type {
    fn drop(&mut self) {
        unsafe {
            wasm_runtime_unload(self.Builder.Module);
            wasm_runtime_deinstantiate(self.Module_instance);
        }
    }
}

impl Module_instance_type {
    /// Lookup a function in the module instance.
    /// Currently, the function signature is ignored.
    pub fn Lookup_function(&self, Function: &Symbol_type) -> Result<Function_type, ()> {
        let Function_instance = unsafe {
            wasm_runtime_lookup_function(
                self.Module_instance,
                Function.Get_name().as_ptr() as *const c_char,
                Function.Get_signature().as_ptr() as *const c_char,
            )
        };
        if Function_instance.is_null() {
            return Err(());
        }

        Ok(Function_type::New(Function_instance))
    }

    pub fn Execute_main(self, mut Arguments: Vec<&str>) -> Result<(), &'static str> {
        unsafe {
            if !wasm_application_execute_main(
                self.Module_instance,
                Arguments.len() as i32,
                Arguments.as_mut_ptr() as *mut *mut i8, // ! : Check the vector lifetime
            ) {
                return Err(
                    CStr::from_ptr(wasm_runtime_get_exception(self.Module_instance))
                        .to_str()
                        .unwrap_or("Unknown error"),
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Destroy, Initialize};
    use super::*;

    #[test]
    fn test_module_builder() {
        Initialize().unwrap();

        let Test_data =
            include_bytes!("../../../Test/target/wasm32-wasi/release/Test.wasm").to_vec();

        Module_builder_type::Load("test", Test_data, None)
            .unwrap()
            .Set_arguments(vec!["argument1", "argument2", "argument3"])
            .Build()
            .unwrap()
            .Execute_main(vec!["argument1", "argument2", "argument3"])
            .unwrap();

        Destroy();
    }
}
