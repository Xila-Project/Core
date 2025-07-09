//! Virtual Machine Manager - Global singleton for WASM runtime management.
//!
//! The Manager provides a centralized interface for initializing the WASM runtime,
//! registering host functions, and executing WASM modules. It maintains a global
//! singleton instance that can be accessed throughout the system.

use core::{ffi::CStr, mem::forget};

use alloc::{string::ToString, vec, vec::Vec};
use file_system::Unique_file_identifier_type;
use synchronization::once_lock::OnceLock;
use wamr_rust_sdk::{
    sys::{wasm_runtime_is_xip_file, wasm_runtime_load, wasm_runtime_register_module},
    value::WasmValue,
};

use crate::{Error_type, Instance_type, Module_type, Registrable_trait, Result_type, Runtime_type};

/// Global singleton instance of the Virtual Machine Manager
static MANAGER_INSTANCE: OnceLock<Manager_type> = OnceLock::new();

/// Initialize the Virtual Machine Manager with a set of registrable host functions.
///
/// This function must be called once before any WASM operations can be performed.
/// It creates a global singleton Manager instance that will persist for the
/// lifetime of the application.
///
/// # Arguments
///
/// * `Registrables` - Array of traits implementing host functions that can be called from WASM
///
/// # Returns
///
/// A static reference to the initialized Manager instance
///
/// # Example
///
/// ```rust,ignore
/// let manager = Initialize(&[&MyHostFunctions]);
/// ```
pub fn Initialize(Registrables: &[&dyn Registrable_trait]) -> &'static Manager_type {
    MANAGER_INSTANCE.get_or_init(|| {
        Manager_type::New(Registrables).expect("Cannot create virtual machine manager")
    });

    get_instance()
}

/// Get a reference to the initialized Virtual Machine Manager instance.
///
/// # Panics
///
/// Panics if called before `Initialize()` has been called.
///
/// # Returns
///
/// A static reference to the Manager instance
pub fn get_instance() -> &'static Manager_type {
    MANAGER_INSTANCE
        .try_get()
        .expect("Cannot get virtual machine manager instance before initialization")
}

/// The Virtual Machine Manager handles WASM runtime lifecycle and module execution.
///
/// This struct encapsulates the WASM runtime and provides high-level operations
/// for executing WASM modules with proper I/O redirection and resource management.
pub struct Manager_type {
    runtime: Runtime_type,
}

unsafe impl Send for Manager_type {}

unsafe impl Sync for Manager_type {}

impl Manager_type {
    /// Create a new Virtual Machine Manager with the given registrable host functions.
    ///
    /// This function initializes the WASM runtime, registers all provided host functions,
    /// and pre-loads any modules that the registrables provide.
    ///
    /// # Arguments
    ///
    /// * `Registrables` - Array of objects implementing host functions and optionally providing WASM modules
    ///
    /// # Returns
    ///
    /// A new Manager instance or an error if initialization fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Runtime initialization fails
    /// - Host function registration fails  
    /// - Module loading fails
    pub fn New(Registrables: &[&dyn Registrable_trait]) -> Result_type<Self> {
        let mut runtime_builder = Runtime_type::builder();

        for Registrable in Registrables {
            runtime_builder = runtime_builder.register(*Registrable);
        }

        let Runtime = runtime_builder.build()?;

        let Manager = Self { runtime: Runtime };

        for Registrable in Registrables {
            if let Some(module_binary) = Registrable.get_binary() {
                Manager.Load_module(module_binary, Registrable.is_XIP(), Registrable.get_name())?;
            }
        }

        Ok(Manager)
    }

    /// Load a WASM module from a buffer for execution.
    ///
    /// This method loads a WASM module into the runtime, either as a regular module
    /// or as an XIP (execute-in-place) module for AOT compiled binaries.
    ///
    /// # Arguments
    ///
    /// * `Buffer` - The WASM module bytecode
    /// * `XIP` - Whether this is an XIP AOT compiled module
    /// * `Name` - Name to register the module under
    ///
    /// # Returns
    ///
    /// Success or an error if loading fails
    ///
    /// # Errors
    ///
    /// Returns an error if the module is not an XIP AOT compiled module or if the module cannot be loaded from the buffer.
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

    /// Execute a WASM module with the specified I/O configuration.
    ///
    /// This is the main entry point for executing WASM modules. It creates a new
    /// module instance, sets up the execution environment with proper I/O redirection,
    /// and calls the module's main function.
    ///
    /// # Arguments
    ///
    /// * `Buffer` - The WASM module bytecode to execute
    /// * `Stack_size` - Stack size in bytes for the WASM instance
    /// * `Standard_in` - File identifier for standard input
    /// * `Standard_out` - File identifier for standard output  
    /// * `Standard_error` - File identifier for standard error
    ///
    /// # Returns
    ///
    /// The return values from the WASM module's main function
    ///
    /// # Errors
    ///
    /// Returns an error if module loading, instantiation, or execution fails
    pub async fn Execute(
        &'static self,
        buffer: Vec<u8>,
        stack_size: usize,
        standard_in: Unique_file_identifier_type,
        standard_out: Unique_file_identifier_type,
        standard_error: Unique_file_identifier_type,
    ) -> Result_type<Vec<WasmValue>> {
        abi::get_instance()
            .call_abi(async || {
                let module = Module_type::From_buffer(
                    &self.runtime,
                    buffer,
                    "module",
                    standard_in,
                    standard_out,
                    standard_error,
                )
                .await?;

                let Instance = Instance_type::New(&self.runtime, &module, stack_size).unwrap();

                let Result = Instance.Call_main(&vec![])?;

                Ok(Result)
            })
            .await
    }
}
