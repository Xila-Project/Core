use core::num::NonZeroUsize;
use core::result::Result;
use core::str::Utf8Error;
use core::{fmt::Display, num::NonZeroU8};
use std::sync::PoisonError;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(Graphics::Error_type),
    Failed_to_create_object,
    UTF_8(Utf8Error),
    Poisoned_lock,
    Failed_to_mount_device(File_system::Error_type),
    Failed_to_get_task_identifier(Task::Error_type),
    Failed_to_execute(Executable::Error_type),
}

impl Error_type {
    pub fn Get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
    }
}

impl From<Executable::Error_type> for Error_type {
    fn from(Error: Executable::Error_type) -> Self {
        Self::Failed_to_execute(Error)
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(Error: Task::Error_type) -> Self {
        Self::Failed_to_get_task_identifier(Error)
    }
}

impl From<File_system::Error_type> for Error_type {
    fn from(Error: File_system::Error_type) -> Self {
        Self::Failed_to_mount_device(Error)
    }
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Self::Poisoned_lock
    }
}

impl From<Utf8Error> for Error_type {
    fn from(Error: Utf8Error) -> Self {
        Self::UTF_8(Error)
    }
}

impl From<Graphics::Error_type> for Error_type {
    fn from(Error: Graphics::Error_type) -> Self {
        Self::Graphics(Error)
    }
}

impl Display for Error_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Graphics(Error) => write!(Formatter, "Graphics: {}", Error),
            Self::Failed_to_create_object => write!(Formatter, "Failed to create object"),
            Self::UTF_8(Error) => write!(Formatter, "UTF-8: {}", Error),
            Self::Poisoned_lock => write!(Formatter, "Poisoned lock"),
            Self::Failed_to_mount_device(Error) => {
                write!(Formatter, "Failed to mount device: {}", Error)
            }
            Self::Failed_to_get_task_identifier(Error) => {
                write!(Formatter, "Failed to get task identifier: {}", Error)
            }
            Self::Failed_to_execute(Error) => write!(Formatter, "Failed to execute: {}", Error),
        }
    }
}

impl From<Error_type> for NonZeroUsize {
    fn from(Error: Error_type) -> Self {
        Error.Get_discriminant().into()
    }
}
