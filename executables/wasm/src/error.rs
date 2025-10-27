use core::{
    fmt,
    num::{NonZeroU8, NonZeroUsize},
};

use xila::{file_system, task, virtual_machine};

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
            Error::MissingArgument(argument) => write!(f, "Missing argument: {}", argument),
            Error::FailedToGetCurrentDirectory => write!(f, "Failed to get current directory"),
            Error::InvalidPath => write!(f, "Invalid path"),
            Error::NotAWasmFile => write!(f, "Not a WASM file"),
            Error::FailedToOpenFile => write!(f, "Failed to open file"),
            Error::FailedToReadFile => write!(f, "Failed to read file"),
            Error::FailedToDuplicateStandard(e) => write!(f, "Failed to duplicate standard: {e:?}"),
            Error::FailedToTransferStandard(e) => write!(f, "Failed to transfer standard: {e:?}"),
            Error::FailedToExecute(e) => write!(f, "Failed to execute: {e:?}"),
            Error::FailedToOpenStandardFile(e) => write!(f, "Failed to open standard file: {e:?}"),
            Error::FailedToSpawnTask(e) => write!(f, "Failed to spawn task: {e:?}"),
        }
    }
}

impl From<Error> for NonZeroUsize {
    fn from(error: Error) -> Self {
        error.get_discriminant().into()
    }
}
