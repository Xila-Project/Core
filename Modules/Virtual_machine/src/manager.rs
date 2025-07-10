//! Virtual Machine Manager - Global singleton for WASM runtime management.

//!
//! The Manager provides a centralized interface for initializing the WASM runtime,
//! registering host functions, and executing WASM modules. It maintains a global
//! singleton instance that can be accessed throughout the system.

use core::{ffi::CStr, mem::forget};

use alloc::{string::ToString, vec, vec::Vec};
use file_system::UniqueFileIdentifier;
use synchronization::once_lock::OnceLock;
use wamr_rust_sdk::{
    sys::{wasm_runtime_is_xip_file, wasm_runtime_load, wasm_runtime_register_module},
    value::WasmValue,
};

use crate::{Error, Instance, Module, Registrable, Result, Runtime};

/// Global singleton instance of the Virtual Machine Manager
static MANAGER_INSTANCE: OnceLock<Manager> = OnceLock::new();

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
pub fn initialize(registrables: &[&dyn Registrable]) -> &'static Manager {
    MANAGER_INSTANCE
        .get_or_init(|| Manager::new(registrables).expect("Cannot create virtual machine manager"));

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
pub fn get_instance() -> &'static Manager {
    MANAGER_INSTANCE
        .try_get()
        .expect("Cannot get virtual machine manager instance before initialization")
}

/// The Virtual Machine Manager handles WASM runtime lifecycle and module execution.
///
/// This struct encapsulates the WASM runtime and provides high-level operations
/// for executing WASM modules with proper I/O redirection and resource management.
pub struct Manager {
    runtime: Runtime,
}

unsafe impl Send for Manager {}

unsafe impl Sync for Manager {}

impl Manager {
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
    pub fn new(registrables: &[&dyn Registrable]) -> Result<Self> {
        let mut runtime_builder = Runtime::builder();

        for registrable in registrables {
            runtime_builder = runtime_builder.register(*registrable);
        }

        let runtime = runtime_builder.build()?;

        let manager = Self { runtime };

        for registrable in registrables {
            if let Some(module_binary) = registrable.get_binary() {
                manager.load_module(module_binary, registrable.is_xip(), registrable.get_name())?;
            }
        }

        Ok(manager)
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
    fn load_module(&self, buffer: &[u8], xip: bool, name: &str) -> Result<()> {
        if unsafe { xip && !wasm_runtime_is_xip_file(buffer.as_ptr(), buffer.len() as u32) } {
            return Err(Error::InvalidModule);
        }

        unsafe {
            let mut buffer = if xip {
                Vec::from_raw_parts(buffer.as_ptr() as *mut u8, buffer.len(), buffer.len())
            } else {
                buffer.to_vec()
            };

            let mut error_buffer = [0_i8; 128];

            let module = wasm_runtime_load(
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                error_buffer.as_mut_ptr(),
                error_buffer.len() as u32,
            );

            if module.is_null() {
                return Err(Error::CompilationError(
                    CStr::from_ptr(error_buffer.as_ptr())
                        .to_string_lossy()
                        .to_string(),
                ));
            }

            if !wasm_runtime_register_module(
                name.as_ptr() as *const i8,
                module,
                error_buffer.as_mut_ptr(),
                error_buffer.len() as u32,
            ) {
                return Err(Error::InternalError);
            }

            forget(buffer);
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
    pub async fn execute(
        &'static self,
        buffer: Vec<u8>,
        stack_size: usize,
        standard_in: UniqueFileIdentifier,
        standard_out: UniqueFileIdentifier,
        standard_error: UniqueFileIdentifier,
    ) -> Result<Vec<WasmValue>> {
        abi::get_instance()
            .call_abi(async || {
                let module = Module::from_buffer(
                    &self.runtime,
                    buffer,
                    "module",
                    standard_in,
                    standard_out,
                    standard_error,
                )
                .await?;

                let instance = Instance::new(&self.runtime, &module, stack_size).unwrap();

                let result = instance.call_main(&vec![])?;

                Ok(result)
            })
            .await
    }
}
