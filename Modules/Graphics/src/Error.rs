use core::fmt::Display;
use std::sync::PoisonError;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Error_type {
    Invalid_reference,
    Already_initialized,
    Failed_to_create_thread,
    Not_initialized,
    Out_of_memory,
    Already_in_use,
    Poisoned_lock,
    Failed_to_register,
    Failed_to_get_resolution,
    Not_registered,
    Not_available,
    Failed_to_create_object,
    Invalid_window_identifier,
}

impl Display for Error_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let String = match self {
            Error_type::Invalid_reference => "Invalid reference",
            Error_type::Already_initialized => "Already initialized",
            Error_type::Failed_to_create_thread => "Failed to create thread",
            Error_type::Not_initialized => "Not initialized",
            Error_type::Out_of_memory => "Out of memory",
            Error_type::Already_in_use => "Already in use",
            Error_type::Poisoned_lock => "Poisoned lock",
            Error_type::Failed_to_register => "Failed to register",
            Error_type::Failed_to_get_resolution => "Failed to get resolution",
            Error_type::Not_registered => "Not registered",
            Error_type::Not_available => "Not available",
            Error_type::Failed_to_create_object => "Failed to create object",
            Error_type::Invalid_window_identifier => "Invalid window identifier",
        };

        write!(Formatter, "{}", String)
    }
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Error_type::Poisoned_lock
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(_: Task::Error_type) -> Self {
        Error_type::Failed_to_create_thread
    }
}
