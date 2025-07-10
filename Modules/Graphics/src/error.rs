use core::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Error {
    InvalidReference,
    AlreadyInitialized,
    NotInitialized,
    OutOfMemory,
    AlreadyInUse,
    FailedToRegister,
    FailedToGetResolution,
    NotRegistered,
    NotAvailable,
    FailedToCreateObject,
    InvalidWindowIdentifier,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let string = match self {
            Error::InvalidReference => "Invalid reference",
            Error::AlreadyInitialized => "Already initialized",
            Error::NotInitialized => "Not initialized",
            Error::OutOfMemory => "Out of memory",
            Error::AlreadyInUse => "Already in use",
            Error::FailedToRegister => "Failed to register",
            Error::FailedToGetResolution => "Failed to get resolution",
            Error::NotRegistered => "Not registered",
            Error::NotAvailable => "Not available",
            Error::FailedToCreateObject => "Failed to create object",
            Error::InvalidWindowIdentifier => "Invalid window identifier",
        };

        write!(formatter, "{string}")
    }
}
