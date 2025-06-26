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

impl From<File_system::Error_type> for Error_type {
    fn from(Error: File_system::Error_type) -> Self {
        Error_type::File_system(Error)
    }
}

impl From<Virtual_file_system::Error_type> for Error_type {
    fn from(Error: Virtual_file_system::Error_type) -> Self {
        Error_type::Virtual_file_system(Error)
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(Error: Task::Error_type) -> Self {
        Error_type::Failed_to_set_environment_variable(Error)
    }
}

impl From<core::str::Utf8Error> for Error_type {
    fn from(Error: core::str::Utf8Error) -> Self {
        Error_type::Invalid_UTF_8(Error)
    }
}

impl From<alloc::ffi::NulError> for Error_type {
    fn from(Error: alloc::ffi::NulError) -> Self {
        Error_type::Null_character_in_string(Error)
    }
}

impl Display for Error_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error_type::Graphics(Error) => write!(Formatter, "Graphics error: {}", Error),
            Error_type::File_system(Error) => write!(Formatter, "File system error: {}", Error),
            Error_type::Virtual_file_system(Error) => {
                write!(Formatter, "Virtual file system error: {}", Error)
            }
            Error_type::Failed_to_create_object => write!(Formatter, "Failed to create object"),
            Error_type::Failed_to_get_child => write!(Formatter, "Failed to get child"),
            Error_type::Failed_to_set_environment_variable(Error) => {
                write!(Formatter, "Failed to set environment variable: {}", Error)
            }
            Error_type::Invalid_UTF_8(Error) => write!(Formatter, "Invalid UTF-8: {}", Error),
            Error_type::Failed_to_set_task_user(Error) => {
                write!(Formatter, "Failed to set task user: {}", Error)
            }
            Error_type::Failed_to_get_current_task_identifier(Error) => {
                write!(
                    Formatter,
                    "Failed to get current task identifier: {}",
                    Error
                )
            }
            Error_type::Failed_to_read_directory(Error) => {
                write!(Formatter, "Failed to read directory: {}", Error)
            }
            Error_type::Failed_to_open_standard_file(Error) => {
                write!(Formatter, "Failed to open standard file: {}", Error)
            }
            Error_type::Null_character_in_string(Error) => {
                write!(Formatter, "Null character in string: {}", Error)
            }
            Error_type::Missing_arguments => write!(Formatter, "Missing arguments"),
        }
    }
}
