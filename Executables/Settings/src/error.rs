use core::{
    fmt::Display,
    num::{NonZeroU8, NonZeroUsize},
};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(graphics::Error_type) = 1,
    File_system(file_system::Error_type),
    Virtual_file_system(virtual_file_system::Error_type),
    Failed_to_create_object,
    Failed_to_get_child,
    Failed_to_set_environment_variable(task::Error_type),
    Invalid_UTF_8(core::str::Utf8Error),
    Failed_to_set_task_user(task::Error_type),
    Failed_to_get_current_task_identifier(task::Error_type),
    Failed_to_read_directory(file_system::Error_type),
    Failed_to_open_standard_file(file_system::Error_type),
    Null_character_in_string(alloc::ffi::NulError),
    Missing_arguments,
    Failed_to_create_UI_element,
    Authentication(authentication::Error_type),
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

impl From<file_system::Error_type> for Error_type {
    fn from(error: file_system::Error_type) -> Self {
        Error_type::File_system(error)
    }
}

impl From<virtual_file_system::Error_type> for Error_type {
    fn from(error: virtual_file_system::Error_type) -> Self {
        Error_type::Virtual_file_system(error)
    }
}

impl From<task::Error_type> for Error_type {
    fn from(error: task::Error_type) -> Self {
        Error_type::Failed_to_set_environment_variable(error)
    }
}

impl From<core::str::Utf8Error> for Error_type {
    fn from(error: core::str::Utf8Error) -> Self {
        Error_type::Invalid_UTF_8(error)
    }
}

impl From<alloc::ffi::NulError> for Error_type {
    fn from(error: alloc::ffi::NulError) -> Self {
        Error_type::Null_character_in_string(error)
    }
}

impl Display for Error_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error_type::Graphics(error) => write!(formatter, "Graphics error: {error}"),
            Error_type::File_system(error) => write!(formatter, "File system error: {error}"),
            Error_type::Virtual_file_system(error) => {
                write!(formatter, "Virtual file system error: {error}")
            }
            Error_type::Failed_to_create_object => write!(formatter, "Failed to create object"),
            Error_type::Failed_to_get_child => write!(formatter, "Failed to get child"),
            Error_type::Failed_to_set_environment_variable(error) => {
                write!(formatter, "Failed to set environment variable: {error}")
            }
            Error_type::Invalid_UTF_8(error) => write!(formatter, "Invalid UTF-8: {error}"),
            Error_type::Failed_to_set_task_user(error) => {
                write!(formatter, "Failed to set task user: {error}")
            }
            Error_type::Failed_to_get_current_task_identifier(error) => {
                write!(formatter, "Failed to get current task identifier: {error}")
            }
            Error_type::Failed_to_read_directory(error) => {
                write!(formatter, "Failed to read directory: {error}")
            }
            Error_type::Failed_to_open_standard_file(error) => {
                write!(formatter, "Failed to open standard file: {error}")
            }
            Error_type::Null_character_in_string(error) => {
                write!(formatter, "Null character in string: {error}")
            }
            Error_type::Missing_arguments => write!(formatter, "Missing arguments"),
            Error_type::Failed_to_create_UI_element => {
                write!(formatter, "Failed to create UI element")
            }
            Error_type::Authentication(error) => write!(formatter, "Authentication error: {error}"),
        }
    }
}
