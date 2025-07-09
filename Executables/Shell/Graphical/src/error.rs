use core::{
    fmt::Display,
    num::{NonZeroU8, NonZeroUsize},
};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(graphics::Error_type) = 1,
    Failed_to_create_object,
    Failed_to_get_child,
    Failed_to_set_environment_variable(task::Error_type),
    Invalid_UTF_8(core::str::Utf8Error),
    Authentication_failed(authentication::Error_type),
    Failed_to_set_task_user(task::Error_type),
    Failed_to_deserialize_shortcut(miniserde::Error),
    Failed_to_get_current_task_identifier(task::Error_type),
    Failed_to_read_shortcut_directory(file_system::Error_type),
    Failed_to_get_shortcut_file_path,
    Failed_to_read_shortcut_file(file_system::Error_type),
    Failed_to_open_standard_file(file_system::Error_type),
    Failed_to_execute_shortcut(executable::Error_type),
    Null_character_in_string(alloc::ffi::NulError),
    Missing_arguments,
    Failed_to_add_shortcut(file_system::Error_type),
}

impl Error_type {
    pub fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
    }
}

impl From<Error_type> for NonZeroUsize {
    fn from(error: Error_type) -> Self {
        error.get_discriminant().into()
    }
}

impl From<graphics::Error_type> for Error_type {
    fn from(error: graphics::Error_type) -> Self {
        Error_type::Graphics(error)
    }
}

impl Display for Error_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Graphics(error) => {
                write!(formatter, "Graphics error: {error}")
            }
            Self::Failed_to_create_object => {
                write!(formatter, "Failed to create object")
            }
            Self::Failed_to_get_child => {
                write!(formatter, "Failed to get child")
            }
            Self::Failed_to_set_environment_variable(error) => {
                write!(formatter, "Failed to set environment variable: {error}")
            }
            Self::Invalid_UTF_8(error) => {
                write!(formatter, "Invalid UTF-8: {error}")
            }
            Self::Authentication_failed(error) => {
                write!(formatter, "Authentication failed: {error}")
            }
            Self::Failed_to_set_task_user(error) => {
                write!(formatter, "Failed to set task user: {error}")
            }
            Self::Failed_to_deserialize_shortcut(error) => {
                write!(formatter, "Failed to deserialize shortcut: {error}")
            }
            Self::Failed_to_get_current_task_identifier(error) => {
                write!(formatter, "Failed to get current task identifier: {error}")
            }
            Self::Failed_to_read_shortcut_directory(error) => {
                write!(formatter, "Failed to read shortcut directory: {error}")
            }
            Self::Failed_to_get_shortcut_file_path => {
                write!(formatter, "Failed to get shortcut file path")
            }
            Self::Failed_to_read_shortcut_file(error) => {
                write!(formatter, "Failed to read shortcut file: {error}")
            }
            Self::Failed_to_open_standard_file(error) => {
                write!(formatter, "Failed to open standard file: {error}")
            }
            Self::Failed_to_execute_shortcut(error) => {
                write!(formatter, "Failed to execute shortcut: {error}")
            }
            Self::Null_character_in_string(error) => {
                write!(formatter, "Null character in string: {error}")
            }
            Self::Missing_arguments => {
                write!(formatter, "Missing arguments")
            }
            Self::Failed_to_add_shortcut(error) => {
                write!(formatter, "Failed to add shortcut: {error}")
            }
        }
    }
}
