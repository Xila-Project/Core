//! Error types and result handling for the Virtual Machine module.
//!
//! This module defines comprehensive error types that can occur during WASM module
//! loading, compilation, instantiation, and execution.

use alloc::string::String;
use core::{
    fmt::{self},
    num::{NonZeroU8, NonZeroUsize},
};
use wamr_rust_sdk::RuntimeError;
use xila::{internationalization::translate, shared, task, time, virtual_file_system};

/// Result type alias for Virtual Machine operations
pub type Result<T> = core::result::Result<T, Error>;

/// Comprehensive error types for Virtual Machine operations
///
/// This enum covers all possible error conditions that can occur during
/// WASM module lifecycle operations, from loading to execution.
#[derive(Debug)]
#[repr(C)]
pub enum Error {
    MissingArgument(&'static str),
    FailedToGetCurrentDirectory,
    InvalidPath,
    NotAWasmFile,
    FailedToOpenFile,
    FailedToReadFile,
    FailedToDuplicateStandard(virtual_file_system::Error),
    FailedToTransferStandard(virtual_file_system::Error),
    FailedToOpenStandardFile(virtual_file_system::Error),
    FailedToSpawnTask(task::Error),

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
    FailedToRegisterFileContext,
}

impl Error {
    pub fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
    }
}

impl core::error::Error for Error {}

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingArgument(argument) => {
                write!(f, translate!("Missing argument: {}"), argument)
            }
            Error::FailedToGetCurrentDirectory => {
                write!(f, translate!("Failed to get current directory"))
            }
            Error::InvalidPath => write!(f, translate!("Invalid path")),
            Error::NotAWasmFile => write!(f, translate!("Not a WASM file")),
            Error::FailedToOpenFile => write!(f, translate!("Failed to open file")),
            Error::FailedToReadFile => write!(f, translate!("Failed to read file")),
            Error::FailedToDuplicateStandard(e) => {
                write!(f, translate!("Failed to duplicate standard: {:?}"), e)
            }
            Error::FailedToTransferStandard(e) => {
                write!(f, translate!("Failed to transfer standard: {:?}"), e)
            }
            Error::FailedToOpenStandardFile(e) => {
                write!(f, translate!("Failed to open standard file: {:?}"), e)
            }
            Error::FailedToSpawnTask(e) => {
                write!(f, translate!("Failed to spawn task: {:?}"), e)
            }
            Error::InvalidPointer => write!(f, translate!("Invalid pointer provided")),
            Error::InvalidUtf8String => write!(f, translate!("Invalid UTF-8 string")),
            Error::SliceConversionFailed(e) => {
                write!(f, translate!("Slice conversion failed: {:?}"), e)
            }
            Error::NotImplemented => write!(f, translate!("Functionality not implemented")),
            Error::InitializationFailure => {
                write!(f, translate!("WASM runtime initialization failed"))
            }
            Error::CompilationError(e) => write!(f, translate!("WASM compilation error: {}"), e),
            Error::ExecutionError(e) => write!(f, translate!("WASM execution error: {}"), e),
            Error::FunctionNotFound => {
                write!(f, translate!("Requested function not found in module"))
            }
            Error::AllocationFailure => write!(f, translate!("Memory allocation failed")),
            Error::FailedToGetTaskInformations(e) => {
                write!(f, translate!("Failed to get task information: {:?}"), e)
            }
            Error::PoisonedLock => write!(f, translate!("Mutex or lock is poisoned")),
            Error::InvalidModule => {
                write!(f, translate!("Invalid WASM module format or structure"))
            }
            Error::InternalError => write!(f, translate!("Internal runtime error")),
            Error::InvalidThreadIdentifier => {
                write!(f, translate!("Invalid thread identifier provided"))
            }
            Error::Time(e) => {
                write!(f, translate!("Time-related error: {:?}"), e)
            }
            Error::FailedToRegisterFileContext => {
                write!(f, translate!("Failed to register file context"))
            }
            Error::InstantiationFailure(e) => {
                write!(f, translate!("WASM instantiation error: {}"), e)
            }
        }
    }
}

impl From<Error> for NonZeroUsize {
    fn from(error: Error) -> Self {
        error.get_discriminant().into()
    }
}
