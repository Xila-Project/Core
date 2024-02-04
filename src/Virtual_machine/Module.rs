use std::{
    ffi::{c_char, c_void, CStr, CString},
    ptr::null_mut,
    sync::RwLock,
    usize,
};

use crate::Declare_native_symbol;

use super::{Allocation_type, Function::Function_type, Pointer_type, Symbol::Symbol_type};

use wamr_sys::*;


#[derive(Debug)]
pub struct Module_builder_type {
    Name: CString,
    Executable: Vec<u8>,
    Module: wasm_module_t,
    Error_buffer: Vec<i8>,
    Stack_size: usize,
    Heap_size: usize,
    Arguments: Vec<CString>,
    Arguments_pointers: Vec<*mut i8>,
}

const Default_stack_size: usize = 8 * 1024;
const Default_heap_size: usize = 8 * 1024;
const Default_error_buffer_size: usize = 128;

Declare_native_symbol!{wasm_runtime_addr_app_to_native, "i"}

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
            Heap_size: Default_heap_size,
            Arguments: vec![],
            Arguments_pointers: vec![],
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
            Module_instance: RwLock::new(Module_instance),
        })
    }

    pub fn Set_arguments(mut self, Arguments: Vec<&str>) -> Self {
        self.Arguments.reserve(Arguments.len() + 1);
        self.Arguments
            .push(CString::new(self.Name.to_str().unwrap()).unwrap());
        for Argument in Arguments {
            self.Arguments.push(CString::new(Argument).unwrap());
        }

        self.Arguments_pointers = self
            .Arguments
            .iter()
            .map(|x| x.as_ptr() as *mut i8)
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
                self.Arguments_pointers.as_mut_ptr() as *mut *mut i8,
                self.Arguments_pointers.len() as i32,
            );
        }
        self
    }
}

/// Module instance type
#[derive(Debug)]
pub struct Module_instance_type {
    /// Previous builder to take ownership of the module
    Builder: Module_builder_type,
    /// Since the module instance is not thread safe, we use a RwLock
    Module_instance: RwLock<wasm_module_inst_t>,
}

impl Drop for Module_instance_type {
    fn drop(&mut self) {
        unsafe {
            wasm_runtime_unload(self.Builder.Module);
            wasm_runtime_deinstantiate(*self.Module_instance.write().unwrap());
        }
    }
}

impl Module_instance_type {
    /// Lookup a function in the module instance.
    /// Currently, the function signature is ignored.
    pub fn Lookup_function(&self, Function: &Symbol_type) -> Result<Function_type, ()> {
        let Function_instance = unsafe {
            wasm_runtime_lookup_function(
                *self.Module_instance.read().unwrap(),
                Function.Get_name().as_ptr() as *const c_char,
                Function.Get_signature().as_ptr() as *const c_char,
            )
        };
        if Function_instance.is_null() {
            return Err(());
        }

        Ok(Function_type::New(Function_instance))
    }

    pub fn Allocate_memory(&self, Size: usize) -> Result<Allocation_type, ()> {
        let mut Native_pointer: *mut c_void = null_mut();
        let Pointer = unsafe {
            wasm_runtime_module_malloc(
                *self.Module_instance.write().unwrap(),
                Size as u32,
                &mut Native_pointer as *mut *mut c_void,
            )
        };
        if Pointer == 0 {
            return Err(());
        }
        Ok(Allocation_type::New(self, Native_pointer, Pointer, Size))
    }

    pub fn Duplicate_memory<T>(&self, Data: &T, Size: usize) -> Result<Allocation_type, ()> {
        let Pointer = unsafe {
            wasm_runtime_module_dup_data(
                *self.Module_instance.write().unwrap(),
                Data as *const T as *const i8,
                Size as u32,
            )
        };
        if Pointer == 0 {
            return Err(());
        }
        Ok(Allocation_type::New(self, null_mut(), Pointer, Size))
    }

    pub fn Free_memory(&self, Pointer: Pointer_type) {
        unsafe {
            wasm_runtime_module_free(*self.Module_instance.write().unwrap(), Pointer);
        }
    }

    pub fn Execute_main(self) -> Result<i32, &'static str> {
        unsafe {
            if !wasm_application_execute_main(
                *self.Module_instance.write().unwrap(),
                0,
                null_mut(), // TODO : Find a way to recover the return value
            ) {
                return Err(CStr::from_ptr(wasm_runtime_get_exception(
                    *self.Module_instance.read().unwrap(),
                ))
                .to_str()
                .unwrap_or("Unknown error"));
            }
        }
        Ok(0)
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

        let Module = Module_builder_type::Load("test", Test_data, None)
            .unwrap()
            .Set_arguments(vec!["argument1", "argument2", "argument3"])
            .Build()
            .unwrap();

        Module.Execute_main().unwrap();

        Destroy();
    }
}
