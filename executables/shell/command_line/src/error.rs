use core::fmt::Display;
use core::num::{NonZeroU16, NonZeroUsize};

use xila::{authentication, task};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
#[repr(u16)]
pub enum Error {
    AuthenticationFailed(authentication::Error) = 1,
    FailedToSetTaskUser(task::Error),
    FailedToSetEnvironmentVariable(task::Error),
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
}

impl Error {
    pub fn get_discriminant(&self) -> NonZeroU16 {
        unsafe { *<*const _>::from(self).cast::<NonZeroU16>() }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::AuthenticationFailed(error) => {
                write!(formatter, "Authentication failed: {error}")
            }
            Error::FailedToSetTaskUser(error) => {
                write!(formatter, "Failed to set task user: {error}")
            }
            Error::FailedToSetEnvironmentVariable(error) => {
                write!(formatter, "Failed to set environment variable: {error}")
            }
            Error::FailedToTokenizeCommandLine => {
                write!(formatter, "Failed to tokenize command line")
            }
            Error::MissingFileNameAfterRedirectOut => {
                write!(formatter, "Missing file name after redirect out")
            }
            Error::MissingFileNameAfterRedirectIn => {
                write!(formatter, "Missing file name after redirect in")
            }
            Error::MissingCommand => write!(formatter, "Missing command"),
            Error::CommandNotFound => write!(formatter, "Command not found"),
            Error::FailedToGetTaskIdentifier => {
                write!(formatter, "Failed to get task identifier")
            }
            Error::InvalidPath => write!(formatter, "Invalid path"),
            Error::FailedToGetPath => {
                write!(formatter, "Failed to get environment variable")
            }
            Error::FailedToExecuteCommand => {
                write!(formatter, "Failed to execute command")
            }
            Error::FailedToJoinTask => write!(formatter, "Failed to join task"),
            Error::InvalidNumberOfArguments => {
                write!(formatter, "Invalid number of arguments")
            }
        }
    }
}

impl From<Error> for NonZeroUsize {
    fn from(error: Error) -> Self {
        error.get_discriminant().into()
    }
}
