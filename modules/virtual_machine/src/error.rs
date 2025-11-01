//! Error types and result handling for the Virtual Machine module.
//!
//! This module defines comprehensive error types that can occur during WASM module
//! loading, compilation, instantiation, and execution.

use alloc::string::String;
use wamr_rust_sdk::RuntimeError;

/// Result type alias for Virtual Machine operations
pub type Result<T> = core::result::Result<T, Error>;

/// Comprehensive error types for Virtual Machine operations
///
/// This enum covers all possible error conditions that can occur during
/// WASM module lifecycle operations, from loading to execution.
#[derive(Debug)]
#[repr(C)]
pub enum Error {
    /// Invalid pointer provided to a function
    InvalidPointer,

    /// String contains invalid UTF-8 sequences
    InvalidUtf8String,

    /// Failed to convert between slice types
    SliceConversionFailed(shared::Error),

    /// Requested functionality is not yet implemented
    NotImplemented,

    /// WASM runtime initialization failed
    InitializationFailure,

    /// WASM module compilation failed with detailed error message
    CompilationError(String),

    /// WASM module instantiation failed with detailed error message
    InstantiationFailure(String),

    /// WASM function execution failed with detailed error message
    ExecutionError(String),

    /// Requested function was not found in the module
    FunctionNotFound,

    /// Memory allocation failed
    AllocationFailure,

    /// Failed to retrieve task information
    FailedToGetTaskInformations(task::Error),

    /// Mutex or lock was poisoned
    PoisonedLock,

    /// Invalid WASM module format or structure
    InvalidModule,

    /// Internal runtime error
    InternalError,

    /// Invalid thread identifier provided
    InvalidThreadIdentifier,

    /// Time-related operation failed
    Time(time::Error),

    /// Failed to transfert file identifiers
    FailedToTransferFileIdentifiers(file_system::Error),
}

impl From<RuntimeError> for Error {
    fn from(error: RuntimeError) -> Self {
        match error {
            RuntimeError::NotImplemented => Error::NotImplemented,
            RuntimeError::InitializationFailure => Error::InitializationFailure,
            RuntimeError::WasmFileFSError(_) => Error::InitializationFailure,
            RuntimeError::CompilationError(e) => Error::CompilationError(e),
            RuntimeError::InstantiationFailure(e) => Error::InstantiationFailure(e),
            RuntimeError::ExecutionError(e) => Error::ExecutionError(e.message),
            RuntimeError::FunctionNotFound => Error::FunctionNotFound,
        }
    }
}
impl From<task::Error> for Error {
    fn from(error: task::Error) -> Self {
        Error::FailedToGetTaskInformations(error)
    }
}
