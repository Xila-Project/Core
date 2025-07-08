use core::fmt::Display;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Error_type {
    Invalid_reference,
    Already_initialized,
    Not_initialized,
    Out_of_memory,
    Already_in_use,
    Failed_to_register,
    Failed_to_get_resolution,
    Not_registered,
    Not_available,
    Failed_to_create_object,
    Invalid_window_identifier,
}

impl Display for Error_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let string = match self {
            Error_type::Invalid_reference => "Invalid reference",
            Error_type::Already_initialized => "Already initialized",
            Error_type::Not_initialized => "Not initialized",
            Error_type::Out_of_memory => "Out of memory",
            Error_type::Already_in_use => "Already in use",
            Error_type::Failed_to_register => "Failed to register",
            Error_type::Failed_to_get_resolution => "Failed to get resolution",
            Error_type::Not_registered => "Not registered",
            Error_type::Not_available => "Not available",
            Error_type::Failed_to_create_object => "Failed to create object",
            Error_type::Invalid_window_identifier => "Invalid window identifier",
        };

        write!(formatter, "{string}")
    }
}
