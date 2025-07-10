use core::{fmt::Display, num::NonZeroU8};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Error {
    NotFound = 1,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddressInUse,
    AddressNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    ResourceBusy,
    Deadlock,
    Interrupted,
    Unsupported,
    UnexpectedEndOfFile,
    OutOfMemory,
    InProgress,
    PoisonnedLock,
    UnsupportedProtocol,
    InvalidIdentifier,
    DuplicateIdentifier,
    Other,
}

impl Error {
    pub const fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { NonZeroU8::new_unchecked(*self as u8) }
    }
}

impl From<Error> for NonZeroU8 {
    fn from(value: Error) -> Self {
        value.get_discriminant()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::NotFound => write!(f, "Not found"),
            Error::PermissionDenied => write!(f, "Permission denied"),
            Error::ConnectionRefused => write!(f, "Connection refused"),
            Error::ConnectionReset => write!(f, "Connection reset"),
            Error::HostUnreachable => write!(f, "Host unreachable"),
            Error::NetworkUnreachable => write!(f, "Network unreachable"),
            Error::ConnectionAborted => write!(f, "Connection aborted"),
            Error::NotConnected => write!(f, "Not connected"),
            Error::AddressInUse => write!(f, "Address in use"),
            Error::AddressNotAvailable => write!(f, "Address not available"),
            Error::NetworkDown => write!(f, "Network down"),
            Error::BrokenPipe => write!(f, "Broken pipe"),
            Error::AlreadyExists => write!(f, "Already exists"),
            Error::WouldBlock => write!(f, "Would block"),
            Error::InvalidInput => write!(f, "Invalid input"),
            Error::InvalidData => write!(f, "Invalid data"),
            Error::TimedOut => write!(f, "Timed out"),
            Error::WriteZero => write!(f, "Write zero"),
            Error::StorageFull => write!(f, "Storage full"),
            Error::ResourceBusy => write!(f, "Resource busy"),
            Error::Deadlock => write!(f, "Deadlock"),
            Error::Interrupted => write!(f, "Interrupted"),
            Error::Unsupported => write!(f, "Unsupported operation"),
            Error::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
            Error::OutOfMemory => write!(f, "Out of memory"),
            Error::InProgress => write!(f, "In progress operation not completed yet"),
            Error::PoisonnedLock => write!(f, "Poisoned lock encountered an error state"),
            Error::UnsupportedProtocol => write!(f, "Unsupported protocol used in operation"),
            Error::InvalidIdentifier => {
                write!(f, "Invalid identifier provided for operation")
            }
            Error::DuplicateIdentifier => {
                write!(f, "Duplicate identifier found in operation")
            }
            Error::Other => write!(f, "Other error occurred"),
        }
    }
}
