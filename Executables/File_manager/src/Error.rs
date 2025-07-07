use core::{
    fmt::Display,
    num::{NonZeroU8, NonZeroUsize},
};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(Graphics::Error_type) = 1,
    File_system(File_system::Error_type),
    Virtual_file_system(Virtual_file_system::Error_type),
    Failed_to_create_object,
    Failed_to_get_child,
    Failed_to_set_environment_variable(Task::Error_type),
    Invalid_UTF_8(core::str::Utf8Error),
    Failed_to_set_task_user(Task::Error_type),
    Failed_to_get_current_task_identifier(Task::Error_type),
    Failed_to_read_directory(File_system::Error_type),
    Failed_to_open_standard_file(File_system::Error_type),
    Null_character_in_string(alloc::ffi::NulError),
    Missing_arguments,
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

impl From<Graphics::Error_type> for Error_type {
    fn from(error: Graphics::Error_type) -> Self {
        Error_type::Graphics(error)
    }
}

impl From<File_system::Error_type> for Error_type {
    fn from(error: File_system::Error_type) -> Self {
        Error_type::File_system(error)
    }
}

impl From<Virtual_file_system::Error_type> for Error_type {
    fn from(error: Virtual_file_system::Error_type) -> Self {
        Error_type::Virtual_file_system(error)
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(error: Task::Error_type) -> Self {
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
            Error_type::Graphics(Error) => write!(formatter, "Graphics error: {Error}"),
            Error_type::File_system(error) => write!(formatter, "File system error: {error}"),
            Error_type::Virtual_file_system(error) => {
                write!(formatter, "Virtual file system error: {error}")
            }
            Error_type::Failed_to_create_object => write!(formatter, "Failed to create object"),
            Error_type::Failed_to_get_child => write!(formatter, "Failed to get child"),
            Error_type::Failed_to_set_environment_variable(error) => {
                write!(formatter, "Failed to set environment variable: {error}")
            }
            Error_type::Invalid_UTF_8(Error) => write!(formatter, "Invalid UTF-8: {Error}"),
            Error_type::Failed_to_set_task_user(error) => {
                write!(formatter, "Failed to set task user: {error}")
            }
            Error_type::Failed_to_get_current_task_identifier(Error) => {
                write!(formatter, "Failed to get current task identifier: {Error}")
            }
            Error_type::Failed_to_read_directory(Error) => {
                write!(formatter, "Failed to read directory: {Error}")
            }
            Error_type::Failed_to_open_standard_file(Error) => {
                write!(formatter, "Failed to open standard file: {Error}")
            }
            Error_type::Null_character_in_string(Error) => {
                write!(formatter, "Null character in string: {Error}")
            }
            Error_type::Missing_arguments => write!(formatter, "Missing arguments"),
        }
    }
}
