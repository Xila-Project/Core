use core::{fmt::Display, result::Result};
use std::num::NonZeroUsize;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u16)]
pub enum Error_type {
    Failed_to_tokenize_command_line = 1,
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
    Todo,
}

impl Display for Error_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error_type::Failed_to_tokenize_command_line => {
                write!(Formatter, "Failed to tokenize command line")
            }
            Error_type::Missing_file_name_after_redirect_out => {
                write!(Formatter, "Missing file name after redirect out")
            }
            Error_type::Missing_file_name_after_redirect_in => {
                write!(Formatter, "Missing file name after redirect in")
            }
            Error_type::Missing_command => write!(Formatter, "Missing command"),
            Error_type::Command_not_found => write!(Formatter, "Command not found"),
            Error_type::Failed_to_get_task_identifier => {
                write!(Formatter, "Failed to get task identifier")
            }
            Error_type::Invalid_path => write!(Formatter, "Invalid path"),
            Error_type::Failed_to_get_path => {
                write!(Formatter, "Failed to get environment variable")
            }
            Error_type::Failed_to_execute_command => {
                write!(Formatter, "Failed to execute command")
            }
            Error_type::Failed_to_join_task => write!(Formatter, "Failed to join task"),
            Error_type::Invalid_number_of_arguments => {
                write!(Formatter, "Invalid number of arguments")
            }
            Error_type::Todo => write!(Formatter, "Todo"),
        }
    }
}

impl From<Error_type> for NonZeroUsize {
    fn from(Error: Error_type) -> Self {
        NonZeroUsize::new(Error as usize).unwrap()
    }
}
