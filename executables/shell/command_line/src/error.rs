use core::fmt::Display;
use core::num::{NonZeroU16, NonZeroUsize};

use alloc::fmt;
use xila::file_system;
use xila::{authentication, internationalization::translate, task};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
#[repr(u16)]
pub enum Error {
    AuthenticationFailed(authentication::Error) = 1,
    FailedToSetTaskUser(task::Error),
    FailedToSetEnvironmentVariable(task::Error),
    FailedToSetCurrentDirectory(task::Error),
    FailedToRemoveEnvironmentVariable(task::Error),
    FailedToReadEnvironmentVariable(task::Error),
    FailedToTokenizeCommandLine,
    MissingFileNameAfterRedirectOut,
    MissingFileNameAfterRedirectIn,
    MissingCommand,
    CommandNotFound,
    FailedToGetTaskIdentifier,
    InvalidPath,
    FailedToGetPath,
    FailedToExecuteCommand,
    FailedToJoinTask,
    InvalidNumberOfArguments,
    FailedToJoinPath,
    FailedToCreateDirectory(file_system::Error),
    FailedToRemoveDirectory(file_system::Error),
    FailedToOpenDirectory(file_system::Error),
    FailedToOpenFile(file_system::Error),
    InvalidArgument,
    FailedToGetMetadata(file_system::Error),
    FailedToReadDirectoryEntry(file_system::Error),
    FormatError,
}

impl Error {
    pub fn get_discriminant(&self) -> NonZeroU16 {
        unsafe { *<*const _>::from(self).cast::<NonZeroU16>() }
    }
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Self {
        Error::FormatError
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::AuthenticationFailed(error) => {
                write!(formatter, translate!("Authentication failed: {}"), error)
            }
            Error::FailedToSetTaskUser(error) => {
                write!(formatter, translate!("Failed to set task user: {}"), error)
            }
            Error::FailedToSetEnvironmentVariable(error) => {
                write!(
                    formatter,
                    translate!("Failed to set environment variable: {}"),
                    error
                )
            }
            Error::FailedToTokenizeCommandLine => {
                write!(formatter, translate!("Failed to tokenize command line"))
            }
            Error::MissingFileNameAfterRedirectOut => {
                write!(
                    formatter,
                    translate!("Missing file name after redirect out")
                )
            }
            Error::MissingFileNameAfterRedirectIn => {
                write!(formatter, translate!("Missing file name after redirect in"))
            }
            Error::MissingCommand => write!(formatter, translate!("Missing command")),
            Error::CommandNotFound => write!(formatter, translate!("Command not found")),
            Error::FailedToGetTaskIdentifier => {
                write!(formatter, translate!("Failed to get task identifier"))
            }
            Error::InvalidPath => write!(formatter, translate!("Invalid path")),
            Error::FailedToGetPath => {
                write!(formatter, translate!("Failed to get environment variable"))
            }
            Error::FailedToExecuteCommand => {
                write!(formatter, translate!("Failed to execute command"))
            }
            Error::FailedToJoinTask => {
                write!(formatter, translate!("Failed to join task"))
            }
            Error::InvalidNumberOfArguments => {
                write!(formatter, translate!("Invalid number of arguments"))
            }
            Error::FailedToRemoveEnvironmentVariable(error) => {
                write!(
                    formatter,
                    translate!("Failed to remove environment variable: {}"),
                    error
                )
            }
            Error::FailedToJoinPath => {
                write!(formatter, translate!("Failed to join path"))
            }
            Error::FailedToCreateDirectory(error) => {
                write!(
                    formatter,
                    translate!("Failed to create directory: {}"),
                    error
                )
            }
            Error::FailedToRemoveDirectory(error) => {
                write!(
                    formatter,
                    translate!("Failed to remove directory: {}"),
                    error
                )
            }
            Error::FailedToOpenDirectory(error) => {
                write!(formatter, translate!("Failed to open directory: {}"), error)
            }
            Error::FailedToOpenFile(error) => {
                write!(formatter, translate!("Failed to open file: {}"), error)
            }
            Error::InvalidArgument => {
                write!(formatter, translate!("Invalid argument"))
            }
            Error::FailedToGetMetadata(error) => {
                write!(formatter, translate!("Failed to get metadata: {}"), error)
            }
            Error::FailedToSetCurrentDirectory(error) => {
                write!(
                    formatter,
                    translate!("Failed to set current directory: {}"),
                    error
                )
            }
            Error::FailedToReadDirectoryEntry(error) => {
                write!(
                    formatter,
                    translate!("Failed to read directory entry: {}"),
                    error
                )
            }
            Error::FormatError => {
                write!(formatter, translate!("Format error"))
            }
            Error::FailedToReadEnvironmentVariable(error) => {
                write!(
                    formatter,
                    translate!("Failed to read environment variable: {}"),
                    error
                )
            }
        }
    }
}

impl From<Error> for NonZeroUsize {
    fn from(error: Error) -> Self {
        error.get_discriminant().into()
    }
}
