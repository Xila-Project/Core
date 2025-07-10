use core::{fmt, num::NonZeroUsize};

#[repr(u8)]
pub enum Error {
    InvalidNumberOfArguments = 1,
    FailedToGetCurrentDirectory,
    InvalidPath,
    NotAWasmFile,
    FailedToOpenFile,
    FailedToReadFile,
    FailedToDuplicateStandard,
    FailedToExecute,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Error::InvalidNumberOfArguments => "Invalid number of arguments",
            Error::FailedToGetCurrentDirectory => "Failed to get current directory",
            Error::InvalidPath => "Invalid path",
            Error::NotAWasmFile => "Not a WASM file",
            Error::FailedToOpenFile => "Failed to open file",
            Error::FailedToReadFile => "Failed to read file",
            Error::FailedToDuplicateStandard => "Failed to duplicate standard",
            Error::FailedToExecute => "Failed to execute",
        };

        write!(f, "{string}")
    }
}

impl From<Error> for NonZeroUsize {
    fn from(error: Error) -> Self {
        unsafe { NonZeroUsize::new_unchecked(error as usize) }
    }
}
