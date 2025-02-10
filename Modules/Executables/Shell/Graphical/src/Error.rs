use core::{fmt::Display, num::NonZeroUsize};
use std::num::NonZeroU8;

pub type Result_type<T> = Result<T, Error_type>;

extern crate alloc;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(Graphics::Error_type) = 1,
    Failed_to_create_object,
    Failed_to_get_child,
    Failed_to_set_environment_variable(Task::Error_type),
    Invalid_UTF_8(core::str::Utf8Error),
    Authentication_failed(Authentication::Error_type),
    Failed_to_set_task_user(Task::Error_type),
    Failed_to_deserialize_shortcut(miniserde::Error),
    Failed_to_get_current_task_identifier(Task::Error_type),
    Failed_to_read_shortcut_directory(File_system::Error_type),
    Failed_to_get_shortcut_file_path,
    Failed_to_read_shortcut_file(File_system::Error_type),
    Failed_to_open_standard_file(File_system::Error_type),
    Failed_to_execute_shortcut(Executable::Error_type),
    Null_character_in_string(alloc::ffi::NulError),
    Missing_arguments,
    Failed_to_add_shortcut(File_system::Error_type),
}

impl Error_type {
    pub fn Get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
    }
}

impl From<Error_type> for NonZeroUsize {
    fn from(Error: Error_type) -> Self {
        Error.Get_discriminant().into()
    }
}

impl From<Graphics::Error_type> for Error_type {
    fn from(Error: Graphics::Error_type) -> Self {
        Error_type::Graphics(Error)
    }
}

impl Display for Error_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Graphics(Error) => {
                write!(Formatter, "Graphics error: {}", Error)
            }
            Self::Failed_to_create_object => {
                write!(Formatter, "Failed to create object")
            }
            Self::Failed_to_get_child => {
                write!(Formatter, "Failed to get child")
            }
            Self::Failed_to_set_environment_variable(Error) => {
                write!(Formatter, "Failed to set environment variable: {}", Error)
            }
            Self::Invalid_UTF_8(Error) => {
                write!(Formatter, "Invalid UTF-8: {}", Error)
            }
            Self::Authentication_failed(Error) => {
                write!(Formatter, "Authentication failed: {}", Error)
            }
            Self::Failed_to_set_task_user(Error) => {
                write!(Formatter, "Failed to set task user: {}", Error)
            }
            Self::Failed_to_deserialize_shortcut(Error) => {
                write!(Formatter, "Failed to deserialize shortcut: {}", Error)
            }
            Self::Failed_to_get_current_task_identifier(Error) => {
                write!(
                    Formatter,
                    "Failed to get current task identifier: {}",
                    Error
                )
            }
            Self::Failed_to_read_shortcut_directory(Error) => {
                write!(Formatter, "Failed to read shortcut directory: {}", Error)
            }
            Self::Failed_to_get_shortcut_file_path => {
                write!(Formatter, "Failed to get shortcut file path")
            }
            Self::Failed_to_read_shortcut_file(Error) => {
                write!(Formatter, "Failed to read shortcut file: {}", Error)
            }
            Self::Failed_to_open_standard_file(Error) => {
                write!(Formatter, "Failed to open standard file: {}", Error)
            }
            Self::Failed_to_execute_shortcut(Error) => {
                write!(Formatter, "Failed to execute shortcut: {}", Error)
            }
            Self::Null_character_in_string(Error) => {
                write!(Formatter, "Null character in string: {}", Error)
            }
            Self::Missing_arguments => {
                write!(Formatter, "Missing arguments")
            }
            Self::Failed_to_add_shortcut(Error) => {
                write!(Formatter, "Failed to add shortcut: {}", Error)
            }
        }
    }
}
