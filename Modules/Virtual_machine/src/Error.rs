#![allow(non_camel_case_types)]

use wamr_rust_sdk::RuntimeError;

pub type Result_type<T> = std::result::Result<T, Error_type>;

#[derive(Debug)]
#[repr(C)]
pub enum Error_type {
    Invalid_pointer,
    Invalid_UTF8_string,
    Slice_conversion_failed(Shared::Error_type),
    Not_implemented,
    Initialization_failure,
    Compilation_error(String),
    Instantiation_failure(String),
    Execution_error(String),
    Function_not_found,
    Allocation_failure,
    Failed_to_get_task_informations(Task::Error_type),
}

impl From<RuntimeError> for Error_type {
    fn from(Error: RuntimeError) -> Self {
        match Error {
            RuntimeError::NotImplemented => Error_type::Not_implemented,
            RuntimeError::InitializationFailure => Error_type::Initialization_failure,
            RuntimeError::WasmFileFSError(_) => Error_type::Initialization_failure,
            RuntimeError::CompilationError(e) => Error_type::Compilation_error(e),
            RuntimeError::InstantiationFailure(e) => Error_type::Instantiation_failure(e),
            RuntimeError::ExecutionError(e) => Error_type::Execution_error(e),
            RuntimeError::FunctionNotFound => Error_type::Function_not_found,
        }
    }
}
