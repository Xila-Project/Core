use core::num::NonZeroUsize;
use core::result::Result;
use core::str::Utf8Error;
use core::{fmt::Display, num::NonZeroU8};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(Graphics::Error_type) = 1,
    Failed_to_create_object,
    UTF_8(Utf8Error),
    Failed_to_mount_device(File_system::Error_type),
    Failed_to_get_task_identifier(Task::Error_type),
    Failed_to_execute(Executable::Error_type),
}

impl Error_type {
    pub fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
    }
}

impl From<Executable::Error_type> for Error_type {
    fn from(error: Executable::Error_type) -> Self {
        Self::Failed_to_execute(error)
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(error: Task::Error_type) -> Self {
        Self::Failed_to_get_task_identifier(error)
    }
}

impl From<File_system::Error_type> for Error_type {
    fn from(error: File_system::Error_type) -> Self {
        Self::Failed_to_mount_device(error)
    }
}

impl From<Utf8Error> for Error_type {
    fn from(error: Utf8Error) -> Self {
        Self::UTF_8(error)
    }
}

impl From<Graphics::Error_type> for Error_type {
    fn from(error: Graphics::Error_type) -> Self {
        Self::Graphics(error)
    }
}

impl Display for Error_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Graphics(Error) => write!(formatter, "Graphics: {Error}"),
            Self::Failed_to_create_object => write!(formatter, "Failed to create object"),
            Self::UTF_8(error) => write!(formatter, "UTF-8: {error}"),
            Self::Failed_to_mount_device(error) => {
                write!(formatter, "Failed to mount device: {error}")
            }
            Self::Failed_to_get_task_identifier(Error) => {
                write!(formatter, "Failed to get task identifier: {Error}")
            }
            Self::Failed_to_execute(Error) => write!(formatter, "Failed to execute: {Error}"),
        }
    }
}

impl From<Error_type> for NonZeroUsize {
    fn from(error: Error_type) -> Self {
        error.get_discriminant().into()
    }
}
