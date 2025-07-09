use core::num::NonZeroUsize;
use core::result::Result;
use core::str::Utf8Error;
use core::{fmt::Display, num::NonZeroU8};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(graphics::Error_type) = 1,
    Failed_to_create_object,
    UTF_8(Utf8Error),
    Failed_to_mount_device(file_system::Error_type),
    Failed_to_get_task_identifier(task::Error_type),
    Failed_to_execute(executable::Error_type),
}

impl Error_type {
    pub fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
    }
}

impl From<executable::Error_type> for Error_type {
    fn from(error: executable::Error_type) -> Self {
        Self::Failed_to_execute(error)
    }
}

impl From<task::Error_type> for Error_type {
    fn from(error: task::Error_type) -> Self {
        Self::Failed_to_get_task_identifier(error)
    }
}

impl From<file_system::Error_type> for Error_type {
    fn from(error: file_system::Error_type) -> Self {
        Self::Failed_to_mount_device(error)
    }
}

impl From<Utf8Error> for Error_type {
    fn from(error: Utf8Error) -> Self {
        Self::UTF_8(error)
    }
}

impl From<graphics::Error_type> for Error_type {
    fn from(error: graphics::Error_type) -> Self {
        Self::Graphics(error)
    }
}

impl Display for Error_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Graphics(error) => write!(formatter, "Graphics: {error}"),
            Self::Failed_to_create_object => write!(formatter, "Failed to create object"),
            Self::UTF_8(error) => write!(formatter, "UTF-8: {error}"),
            Self::Failed_to_mount_device(error) => {
                write!(formatter, "Failed to mount device: {error}")
            }
            Self::Failed_to_get_task_identifier(error) => {
                write!(formatter, "Failed to get task identifier: {error}")
            }
            Self::Failed_to_execute(error) => write!(formatter, "Failed to execute: {error}"),
        }
    }
}

impl From<Error_type> for NonZeroUsize {
    fn from(error: Error_type) -> Self {
        error.get_discriminant().into()
    }
}
