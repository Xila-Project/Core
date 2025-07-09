//! Error types and result handling for the Virtual Machine module.
//!
//! This module defines comprehensive error types that can occur during WASM module
//! loading, compilation, instantiation, and execution.

#![allow(non_camel_case_types)]

use alloc::string::String;
use wamr_rust_sdk::RuntimeError;

/// Result type alias for Virtual Machine operations
pub type Result_type<T> = core::result::Result<T, Error_type>;

/// Comprehensive error types for Virtual Machine operations
///
/// This enum covers all possible error conditions that can occur during
/// WASM module lifecycle operations, from loading to execution.
#[derive(Debug)]
#[repr(C)]
pub enum Error_type {
    /// Invalid pointer provided to a function
    Invalid_pointer,

    /// String contains invalid UTF-8 sequences
    Invalid_UTF8_string,

    /// Failed to convert between slice types
    Slice_conversion_failed(shared::Error_type),

    /// Requested functionality is not yet implemented
    Not_implemented,

    /// WASM runtime initialization failed
    Initialization_failure,

    /// WASM module compilation failed with detailed error message
    Compilation_error(String),

    /// WASM module instantiation failed with detailed error message
    Instantiation_failure(String),

    /// WASM function execution failed with detailed error message
    Execution_error(String),

    /// Requested function was not found in the module
    Function_not_found,

    /// Memory allocation failed
    Allocation_failure,

    /// Failed to retrieve task information
    Failed_to_get_task_informations(task::Error_type),

    /// Mutex or lock was poisoned
    Poisoned_lock,

    /// Invalid WASM module format or structure
    Invalid_module,

    /// Internal runtime error
    Internal_error,

    /// Invalid thread identifier provided
    Invalid_thread_identifier,

    /// Time-related operation failed
    Time(time::Error_type),
}

impl From<RuntimeError> for Error_type {
    fn from(error: RuntimeError) -> Self {
        match error {
            RuntimeError::NotImplemented => Error_type::Not_implemented,
            RuntimeError::InitializationFailure => Error_type::Initialization_failure,
            RuntimeError::WasmFileFSError(_) => Error_type::Initialization_failure,
            RuntimeError::CompilationError(e) => Error_type::Compilation_error(e),
            RuntimeError::InstantiationFailure(e) => Error_type::Instantiation_failure(e),
            RuntimeError::ExecutionError(e) => Error_type::Execution_error(e.message),
            RuntimeError::FunctionNotFound => Error_type::Function_not_found,
        }
    }
}
impl From<task::Error_type> for Error_type {
    fn from(error: task::Error_type) -> Self {
        Error_type::Failed_to_get_task_informations(error)
    }
}
