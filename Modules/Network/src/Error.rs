use std::{num::NonZeroU8, sync::PoisonError};

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
    pub const fn Get_discriminant(&self) -> NonZeroU8 {
        unsafe { NonZeroU8::new_unchecked(*self as u8) }
    }
}

impl From<Error_type> for NonZeroU8 {
    fn from(Value: Error_type) -> Self {
        Value.Get_discriminant()
    }
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Self::Poisonned_lock
    }
}
