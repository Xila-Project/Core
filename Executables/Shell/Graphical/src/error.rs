use core::{
    fmt::Display,
    num::{NonZeroU8, NonZeroUsize},
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error {
    Graphics(graphics::Error) = 1,
    FailedToCreateObject,
    FailedToGetChild,
    FailedToSetEnvironmentVariable(task::Error),
    InvalidUtf8(core::str::Utf8Error),
    AuthenticationFailed(authentication::Error),
    FailedToSetTaskUser(task::Error),
    FailedToDeserializeShortcut(miniserde::Error),
    FailedToGetCurrentTaskIdentifier(task::Error),
    FailedToReadShortcutDirectory(file_system::Error),
    FailedToGetShortcutFilePath,
    FailedToReadShortcutFile(file_system::Error),
    FailedToOpenStandardFile(file_system::Error),
    FailedToExecuteShortcut(executable::Error),
    NullCharacterInString(alloc::ffi::NulError),
    MissingArguments,
    FailedToAddShortcut(file_system::Error),
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

impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Graphics(error) => {
                write!(formatter, "Graphics error: {error}")
            }
            Self::FailedToCreateObject => {
                write!(formatter, "Failed to create object")
            }
            Self::FailedToGetChild => {
                write!(formatter, "Failed to get child")
            }
            Self::FailedToSetEnvironmentVariable(error) => {
                write!(formatter, "Failed to set environment variable: {error}")
            }
            Self::InvalidUtf8(error) => {
                write!(formatter, "Invalid UTF-8: {error}")
            }
            Self::AuthenticationFailed(error) => {
                write!(formatter, "Authentication failed: {error}")
            }
            Self::FailedToSetTaskUser(error) => {
                write!(formatter, "Failed to set task user: {error}")
            }
            Self::FailedToDeserializeShortcut(error) => {
                write!(formatter, "Failed to deserialize shortcut: {error}")
            }
            Self::FailedToGetCurrentTaskIdentifier(error) => {
                write!(formatter, "Failed to get current task identifier: {error}")
            }
            Self::FailedToReadShortcutDirectory(error) => {
                write!(formatter, "Failed to read shortcut directory: {error}")
            }
            Self::FailedToGetShortcutFilePath => {
                write!(formatter, "Failed to get shortcut file path")
            }
            Self::FailedToReadShortcutFile(error) => {
                write!(formatter, "Failed to read shortcut file: {error}")
            }
            Self::FailedToOpenStandardFile(error) => {
                write!(formatter, "Failed to open standard file: {error}")
            }
            Self::FailedToExecuteShortcut(error) => {
                write!(formatter, "Failed to execute shortcut: {error}")
            }
            Self::NullCharacterInString(error) => {
                write!(formatter, "Null character in string: {error}")
            }
            Self::MissingArguments => {
                write!(formatter, "Missing arguments")
            }
            Self::FailedToAddShortcut(error) => {
                write!(formatter, "Failed to add shortcut: {error}")
            }
        }
    }
}
