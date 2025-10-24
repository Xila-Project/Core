use core::{
    fmt::Display,
    num::{NonZeroU8, NonZeroUsize},
};
use xila::{file_system, graphics, task, virtual_file_system};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error {
    Graphics(graphics::Error) = 1,
    FileSystem(file_system::Error),
    VirtualFileSystem(virtual_file_system::Error),
    FailedToCreateObject,
    FailedToGetChild,
    FailedToSetEnvironmentVariable(task::Error),
    InvalidUtf8(core::str::Utf8Error),
    FailedToSetTaskUser(task::Error),
    FailedToGetCurrentTaskIdentifier(task::Error),
    FailedToReadDirectory(file_system::Error),
    FailedToOpenStandardFile(file_system::Error),
    NullCharacterInString(alloc::ffi::NulError),
    MissingArguments,
}

impl Error {
    pub fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
    }
}

impl From<Error> for NonZeroUsize {
    fn from(error: Error) -> Self {
        error.get_discriminant().into()
    }
}

impl From<graphics::Error> for Error {
    fn from(error: graphics::Error) -> Self {
        Error::Graphics(error)
    }
}

impl From<file_system::Error> for Error {
    fn from(error: file_system::Error) -> Self {
        Error::FileSystem(error)
    }
}

impl From<virtual_file_system::Error> for Error {
    fn from(error: virtual_file_system::Error) -> Self {
        Error::VirtualFileSystem(error)
    }
}

impl From<task::Error> for Error {
    fn from(error: task::Error) -> Self {
        Error::FailedToSetEnvironmentVariable(error)
    }
}

impl From<core::str::Utf8Error> for Error {
    fn from(error: core::str::Utf8Error) -> Self {
        Error::InvalidUtf8(error)
    }
}

impl From<alloc::ffi::NulError> for Error {
    fn from(error: alloc::ffi::NulError) -> Self {
        Error::NullCharacterInString(error)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Graphics(error) => write!(formatter, "Graphics error: {error}"),
            Error::FileSystem(error) => write!(formatter, "File system error: {error}"),
            Error::VirtualFileSystem(error) => {
                write!(formatter, "Virtual file system error: {error}")
            }
            Error::FailedToCreateObject => write!(formatter, "Failed to create object"),
            Error::FailedToGetChild => write!(formatter, "Failed to get child"),
            Error::FailedToSetEnvironmentVariable(error) => {
                write!(formatter, "Failed to set environment variable: {error}")
            }
            Error::InvalidUtf8(error) => write!(formatter, "Invalid UTF-8: {error}"),
            Error::FailedToSetTaskUser(error) => {
                write!(formatter, "Failed to set task user: {error}")
            }
            Error::FailedToGetCurrentTaskIdentifier(error) => {
                write!(formatter, "Failed to get current task identifier: {error}")
            }
            Error::FailedToReadDirectory(error) => {
                write!(formatter, "Failed to read directory: {error}")
            }
            Error::FailedToOpenStandardFile(error) => {
                write!(formatter, "Failed to open standard file: {error}")
            }
            Error::NullCharacterInString(error) => {
                write!(formatter, "Null character in string: {error}")
            }
            Error::MissingArguments => write!(formatter, "Missing arguments"),
        }
    }
}
