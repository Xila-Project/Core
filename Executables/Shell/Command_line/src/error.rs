use core::num::{NonZeroU16, NonZeroUsize};
use core::{fmt::Display, result::Result};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u16)]
pub enum Error_type {
    Authentication_failed(Authentication::Error_type) = 1,
    Failed_to_set_task_user(Task::Error_type),
    Failed_to_set_environment_variable(Task::Error_type),
    Failed_to_tokenize_command_line,
    Missing_file_name_after_redirect_out,
    Missing_file_name_after_redirect_in,
    Missing_command,
    Command_not_found,
    Failed_to_get_task_identifier,
    Invalid_path,
    Failed_to_get_path,
    Failed_to_execute_command,
    Failed_to_join_task,
    Invalid_number_of_arguments,
}

impl Error_type {
    pub fn get_discriminant(&self) -> NonZeroU16 {
        unsafe { *<*const _>::from(self).cast::<NonZeroU16>() }
    }
}

impl Display for Error_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error_type::Authentication_failed(error) => {
                write!(formatter, "Authentication failed: {error}")
            }
            Error_type::Failed_to_set_task_user(error) => {
                write!(formatter, "Failed to set task user: {error}")
            }
            Error_type::Failed_to_set_environment_variable(error) => {
                write!(formatter, "Failed to set environment variable: {error}")
            }
            Error_type::Failed_to_tokenize_command_line => {
                write!(formatter, "Failed to tokenize command line")
            }
            Error_type::Missing_file_name_after_redirect_out => {
                write!(formatter, "Missing file name after redirect out")
            }
            Error_type::Missing_file_name_after_redirect_in => {
                write!(formatter, "Missing file name after redirect in")
            }
            Error_type::Missing_command => write!(formatter, "Missing command"),
            Error_type::Command_not_found => write!(formatter, "Command not found"),
            Error_type::Failed_to_get_task_identifier => {
                write!(formatter, "Failed to get task identifier")
            }
            Error_type::Invalid_path => write!(formatter, "Invalid path"),
            Error_type::Failed_to_get_path => {
                write!(formatter, "Failed to get environment variable")
            }
            Error_type::Failed_to_execute_command => {
                write!(formatter, "Failed to execute command")
            }
            Error_type::Failed_to_join_task => write!(formatter, "Failed to join task"),
            Error_type::Invalid_number_of_arguments => {
                write!(formatter, "Invalid number of arguments")
            }
        }
    }
}

impl From<Error_type> for NonZeroUsize {
    fn from(error: Error_type) -> Self {
        error.get_discriminant().into()
    }
}
