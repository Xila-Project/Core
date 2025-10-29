use core::{
    fmt::Display,
    num::{NonZeroU8, NonZeroUsize},
};

pub type Result<T> = core::result::Result<T, Error>;

use xila::{authentication, executable, file_system, graphics, task};

use crate::translations;

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
                write!(formatter, translations::error__graphics!(), error)
            }
            Self::FailedToCreateObject => {
                write!(formatter, translations::error__failed_to_create_object!())
            }
            Self::FailedToGetChild => {
                write!(formatter, translations::error__failed_to_get_child!())
            }
            Self::FailedToSetEnvironmentVariable(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_set_environment_variable!(),
                    error
                )
            }
            Self::InvalidUtf8(error) => {
                write!(formatter, translations::error__invalid_utf8!(), error)
            }
            Self::AuthenticationFailed(error) => {
                write!(
                    formatter,
                    translations::error__authentication_failed!(),
                    error
                )
            }
            Self::FailedToSetTaskUser(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_set_task_user!(),
                    error
                )
            }
            Self::FailedToDeserializeShortcut(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_deserialize_shortcut!(),
                    error
                )
            }
            Self::FailedToGetCurrentTaskIdentifier(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_get_current_task_identifier!(),
                    error
                )
            }
            Self::FailedToReadShortcutDirectory(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_read_shortcut_directory!(),
                    error
                )
            }
            Self::FailedToGetShortcutFilePath => {
                write!(
                    formatter,
                    translations::error__failed_to_get_shortcut_file_path!()
                )
            }
            Self::FailedToReadShortcutFile(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_read_shortcut_file!(),
                    error
                )
            }
            Self::FailedToOpenStandardFile(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_open_standard_file!(),
                    error
                )
            }
            Self::FailedToExecuteShortcut(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_execute_shortcut!(),
                    error
                )
            }
            Self::NullCharacterInString(error) => {
                write!(
                    formatter,
                    translations::error__null_character_in_string!(),
                    error
                )
            }
            Self::MissingArguments => {
                write!(formatter, translations::error__missing_arguments!())
            }
            Self::FailedToAddShortcut(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_add_shortcut!(),
                    error
                )
            }
        }
    }
}
