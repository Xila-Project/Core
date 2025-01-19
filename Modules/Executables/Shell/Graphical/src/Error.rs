use core::{fmt::Display, num::NonZeroUsize};
use std::num::NonZeroU8;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Error_type {
    Graphics(Graphics::Error_type) = 1,
    Failed_to_create_object,
    Failed_to_set_environment_variable(Task::Error_type),
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
        let String = match self {
            Error_type::Graphics(Error) => "Graphics: ".to_string() + &Error.to_string(),
            Error_type::Failed_to_create_object => "Failed to create window".to_string(),
            Error_type::Failed_to_set_environment_variable(Error) => {
                "Failed to set environment variable: ".to_string() + &Error.to_string()
            }
        };

        write!(Formatter, "{}", String)
    }
}
