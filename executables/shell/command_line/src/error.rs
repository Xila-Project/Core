use core::fmt::Display;
use core::num::{NonZeroU16, NonZeroUsize};

use xila::{authentication, task};

use crate::translations;

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
                write!(
                    formatter,
                    translations::error__authentication_failed!(),
                    error
                )
            }
            Error::FailedToSetTaskUser(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_set_task_user!(),
                    error
                )
            }
            Error::FailedToSetEnvironmentVariable(error) => {
                write!(
                    formatter,
                    translations::error__failed_to_set_environment_variable!(),
                    error
                )
            }
            Error::FailedToTokenizeCommandLine => {
                write!(
                    formatter,
                    translations::error__failed_to_tokenize_command_line!()
                )
            }
            Error::MissingFileNameAfterRedirectOut => {
                write!(
                    formatter,
                    translations::error__missing_file_name_after_redirect_out!()
                )
            }
            Error::MissingFileNameAfterRedirectIn => {
                write!(
                    formatter,
                    translations::error__missing_file_name_after_redirect_in!()
                )
            }
            Error::MissingCommand => write!(formatter, translations::error__missing_command!()),
            Error::CommandNotFound => write!(formatter, translations::error__command_not_found!()),
            Error::FailedToGetTaskIdentifier => {
                write!(
                    formatter,
                    translations::error__failed_to_get_task_identifier!()
                )
            }
            Error::InvalidPath => write!(formatter, translations::error__invalid_path!()),
            Error::FailedToGetPath => {
                write!(formatter, translations::error__failed_to_get_path!())
            }
            Error::FailedToExecuteCommand => {
                write!(formatter, translations::error__failed_to_execute_command!())
            }
            Error::FailedToJoinTask => {
                write!(formatter, translations::error__failed_to_join_task!())
            }
            Error::InvalidNumberOfArguments => {
                write!(
                    formatter,
                    translations::error__invalid_number_of_arguments!()
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
