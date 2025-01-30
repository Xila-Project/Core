use core::{fmt::Display, num::NonZeroUsize};
use std::num::NonZeroU8;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(Graphics::Error_type) = 1,
    Failed_to_create_object,
    Failed_to_set_environment_variable(Task::Error_type),
    Invalid_UTF_8(core::str::Utf8Error),
    Authentication_failed(Authentication::Error_type),
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
            Self::Failed_to_set_environment_variable(Error) => {
                write!(Formatter, "Failed to set environment variable: {}", Error)
            }
            Self::Invalid_UTF_8(Error) => {
                write!(Formatter, "Invalid UTF-8: {}", Error)
            }
            Self::Authentication_failed(Error) => {
                write!(Formatter, "Authentication failed: {}", Error)
            }
        }
    }
}
