use std::sync::PoisonError;

use lvgl::{DisplayError, LvError};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Error_type {
    Invalid_reference,
    Already_initialized,
    Not_initialized,
    Out_of_memory,
    Already_in_use,
    Poisoned_lock,
    Failed_to_register,
    Not_registered,
    Not_available,
}

impl From<DisplayError> for Error_type {
    fn from(Error: DisplayError) -> Self {
        match Error {
            DisplayError::NotAvailable => Error_type::Not_available,
            DisplayError::FailedToRegister => Error_type::Failed_to_register,
            DisplayError::NotRegistered => Error_type::Not_registered,
        }
    }
}

impl From<LvError> for Error_type {
    fn from(Error: LvError) -> Self {
        match Error {
            LvError::InvalidReference => Error_type::Invalid_reference,
            LvError::Uninitialized => Error_type::Not_initialized,
            LvError::LvOOMemory => Error_type::Out_of_memory,
            LvError::AlreadyInUse => Error_type::Already_in_use,
        }
    }
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Error_type::Poisoned_lock
    }
}
