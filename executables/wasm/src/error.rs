use core::{
    fmt,
    num::{NonZeroU8, NonZeroUsize},
};

use xila::{file_system, task, virtual_machine};

use crate::translations;

#[repr(u8)]
pub enum Error {
    MissingArgument(&'static str) = 1,
    FailedToGetCurrentDirectory,
    InvalidPath,
    NotAWasmFile,
    FailedToOpenFile,
    FailedToReadFile,
    FailedToDuplicateStandard(file_system::Error),
    FailedToTransferStandard(file_system::Error),
    FailedToExecute(virtual_machine::Error),
    FailedToOpenStandardFile(file_system::Error),
    FailedToSpawnTask(task::Error),
}

impl Error {
    pub fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingArgument(argument) => {
                write!(f, translations::error__missing_argument!(), argument)
            }
            Error::FailedToGetCurrentDirectory => {
                write!(f, translations::error__failed_to_get_current_directory!())
            }
            Error::InvalidPath => write!(f, translations::error__invalid_path!()),
            Error::NotAWasmFile => write!(f, translations::error__not_a_wasm_file!()),
            Error::FailedToOpenFile => write!(f, translations::error__failed_to_open_file!()),
            Error::FailedToReadFile => write!(f, translations::error__failed_to_read_file!()),
            Error::FailedToDuplicateStandard(e) => {
                write!(f, translations::error__failed_to_duplicate_standard!(), e)
            }
            Error::FailedToTransferStandard(e) => {
                write!(f, translations::error__failed_to_transfer_standard!(), e)
            }
            Error::FailedToExecute(e) => write!(f, translations::error__failed_to_execute!(), e),
            Error::FailedToOpenStandardFile(e) => {
                write!(f, translations::error__failed_to_open_standard_file!(), e)
            }
            Error::FailedToSpawnTask(e) => {
                write!(f, translations::error__failed_to_spawn_task!(), e)
            }
        }
    }
}

impl From<Error> for NonZeroUsize {
    fn from(error: Error) -> Self {
        error.get_discriminant().into()
    }
}
