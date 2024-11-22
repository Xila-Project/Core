use std::{ffi::CStr, mem::forget, sync::OnceLock};

use wamr_rust_sdk::{
    sys::{wasm_runtime_is_xip_file, wasm_runtime_load, wasm_runtime_register_module},
    value::WasmValue,
};
use File_system::Unique_file_identifier_type;

use crate::{Error_type, Instance_type, Module_type, Registrable_trait, Result_type, Runtime_type};

static Manager_instance: OnceLock<Manager_type> = OnceLock::new();

pub fn Initialize(Registrables: &[&dyn Registrable_trait]) -> &'static Manager_type {
    Manager_instance.get_or_init(|| {
        Manager_type::New(Registrables).expect("Cannot create virtual machine manager")
    });

    Get_instance()
}

pub fn Get_instance() -> &'static Manager_type {
    Manager_instance
        .get()
        .expect("Cannot get virtual machine manager instance before initialization")
}

pub struct Manager_type {
    Runtime: Runtime_type,
}

unsafe impl Send for Manager_type {}

unsafe impl Sync for Manager_type {}

impl Manager_type {
    pub fn New(Registrables: &[&dyn Registrable_trait]) -> Result_type<Self> {
        let mut Runtime_builder = Runtime_type::Builder();

        for Registrable in Registrables {
            Runtime_builder = Runtime_builder.Register(*Registrable);
        }

        let Runtime = Runtime_builder.Build()?;

        let Manager = Self { Runtime };

        for Registrable in Registrables {
            if let Some(Module_binary) = Registrable.Get_binary() {
                Manager.Load_module(Module_binary, Registrable.Is_XIP(), Registrable.Get_name())?;
            }
        }

        Ok(Manager)
    }

    /// Load a module from a buffer for execution in place.
    ///
    /// # Errors
    ///
    /// If the module is not an XIP AOT compiled module or if the module cannot be loaded from the buffer.
    fn Load_module(&self, Buffer: &[u8], XIP: bool, Name: &str) -> Result_type<()> {
        if unsafe { XIP && !wasm_runtime_is_xip_file(Buffer.as_ptr(), Buffer.len() as u32) } {
            return Err(Error_type::Invalid_module);
        }

        unsafe {
            let mut Buffer = if XIP {
                Vec::from_raw_parts(Buffer.as_ptr() as *mut u8, Buffer.len(), Buffer.len())
            } else {
                Buffer.to_vec()
            };

            let mut Error_buffer = [0_i8; 128];

            let Module = wasm_runtime_load(
                Buffer.as_mut_ptr(),
                Buffer.len() as u32,
                Error_buffer.as_mut_ptr(),
                Error_buffer.len() as u32,
            );

            if Module.is_null() {
                return Err(Error_type::Compilation_error(
                    CStr::from_ptr(Error_buffer.as_ptr())
                        .to_string_lossy()
                        .to_string(),
                ));
            }

            if !wasm_runtime_register_module(
                Name.as_ptr() as *const i8,
                Module,
                Error_buffer.as_mut_ptr(),
                Error_buffer.len() as u32,
            ) {
                return Err(Error_type::Internal_error);
            }

            forget(Buffer);
        }

        Ok(())
    }

    pub fn Instantiate(
        &'static self,
        Buffer: Vec<u8>,
        Stack_size: usize,
        Standard_in: Unique_file_identifier_type,
        Standard_out: Unique_file_identifier_type,
        Standard_error: Unique_file_identifier_type,
    ) -> Result_type<WasmValue> {
        let Module = Module_type::From_buffer(
            &self.Runtime,
            Buffer,
            "module",
            Standard_in,
            Standard_out,
            Standard_error,
        )?;

        let Instance = Instance_type::New(&self.Runtime, &Module, Stack_size).unwrap();

        let Result = Instance.Call_main(&vec![])?;

        Ok(Result)
    }
}
