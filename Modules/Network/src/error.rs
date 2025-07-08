use core::{fmt::Display, num::NonZeroU8};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Error_type {
    Not_found = 1,
    Permission_denied,
    Connection_refused,
    Connection_reset,
    Host_unreachable,
    Network_unreachable,
    Connection_aborted,
    Not_connected,
    Address_in_use,
    Address_not_available,
    Network_down,
    Broken_pipe,
    Already_exists,
    Would_block,
    Invalid_input,
    Invalid_data,
    Timed_out,
    Write_zero,
    Storage_full,
    Resource_busy,
    Deadlock,
    Interrupted,
    Unsupported,
    Unexpected_end_of_file,
    Out_of_memory,
    In_progress,
    Poisonned_lock,
    Unsupported_protocol,
    Invalid_identifier,
    Duplicate_identifier,
    Other,
}

impl Error_type {
    pub const fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { NonZeroU8::new_unchecked(*self as u8) }
    }
}

impl From<Error_type> for NonZeroU8 {
    fn from(value: Error_type) -> Self {
        value.get_discriminant()
    }
}

impl Display for Error_type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error_type::Not_found => write!(f, "Not found"),
            Error_type::Permission_denied => write!(f, "Permission denied"),
            Error_type::Connection_refused => write!(f, "Connection refused"),
            Error_type::Connection_reset => write!(f, "Connection reset"),
            Error_type::Host_unreachable => write!(f, "Host unreachable"),
            Error_type::Network_unreachable => write!(f, "Network unreachable"),
            Error_type::Connection_aborted => write!(f, "Connection aborted"),
            Error_type::Not_connected => write!(f, "Not connected"),
            Error_type::Address_in_use => write!(f, "Address in use"),
            Error_type::Address_not_available => write!(f, "Address not available"),
            Error_type::Network_down => write!(f, "Network down"),
            Error_type::Broken_pipe => write!(f, "Broken pipe"),
            Error_type::Already_exists => write!(f, "Already exists"),
            Error_type::Would_block => write!(f, "Would block"),
            Error_type::Invalid_input => write!(f, "Invalid input"),
            Error_type::Invalid_data => write!(f, "Invalid data"),
            Error_type::Timed_out => write!(f, "Timed out"),
            Error_type::Write_zero => write!(f, "Write zero"),
            Error_type::Storage_full => write!(f, "Storage full"),
            Error_type::Resource_busy => write!(f, "Resource busy"),
            Error_type::Deadlock => write!(f, "Deadlock"),
            Error_type::Interrupted => write!(f, "Interrupted"),
            Error_type::Unsupported => write!(f, "Unsupported operation"),
            Error_type::Unexpected_end_of_file => write!(f, "Unexpected end of file"),
            Error_type::Out_of_memory => write!(f, "Out of memory"),
            Error_type::In_progress => write!(f, "In progress operation not completed yet"),
            Error_type::Poisonned_lock => write!(f, "Poisoned lock encountered an error state"),
            Error_type::Unsupported_protocol => write!(f, "Unsupported protocol used in operation"),
            Error_type::Invalid_identifier => {
                write!(f, "Invalid identifier provided for operation")
            }
            Error_type::Duplicate_identifier => {
                write!(f, "Duplicate identifier found in operation")
            }
            Error_type::Other => write!(f, "Other error occurred"),
        }
    }
}
