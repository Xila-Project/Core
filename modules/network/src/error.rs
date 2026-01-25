use core::{fmt::Display, num::NonZeroU8};
use smoltcp::socket::{dns, icmp, udp};

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
    Pending,
    UnsupportedProtocol,
    InvalidIdentifier,
    DuplicateIdentifier,
    FailedToGenerateSeed(file_system::Error),
    FailedToSpawnNetworkTask(task::Error),
    // - DNS
    InvalidName,
    NameTooLong,
    Failed,
    // - Accept / Connect
    InvalidState,
    InvalidPort,
    NoRoute,
    // Udp
    Truncated,
    SocketNotBound,
    PacketTooLarge,
    InvalidEndpoint,

    FailedToMountDevice(virtual_file_system::Error),

    NoFreeSlot,

    Other,
}

impl core::error::Error for Error {}

impl Error {
    pub const fn get_discriminant(&self) -> NonZeroU8 {
        unsafe { *(self as *const Self as *const NonZeroU8) }
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
            Error::Pending => write!(f, "In progress operation not completed yet"),
            Error::UnsupportedProtocol => write!(f, "Unsupported protocol used in operation"),
            Error::InvalidIdentifier => {
                write!(f, "Invalid identifier provided for operation")
            }
            Error::DuplicateIdentifier => {
                write!(f, "Duplicate identifier found in operation")
            }
            Error::FailedToGenerateSeed(e) => {
                write!(f, "Failed to generate seed: {}", e)
            }
            Error::FailedToSpawnNetworkTask(e) => {
                write!(f, "Failed to spawn network task: {}", e)
            }
            Error::InvalidName => write!(f, "Invalid name"),
            Error::NameTooLong => write!(f, "Name too long"),
            Error::Failed => write!(f, "Failed"),
            Error::InvalidState => write!(f, "Invalid state for operation"),
            Error::InvalidPort => write!(f, "Invalid port specified"),
            Error::NoRoute => write!(f, "No route to host"),
            Error::Truncated => write!(f, "Truncated packet received"),
            Error::SocketNotBound => write!(f, "Socket not bound"),
            Error::PacketTooLarge => write!(f, "Packet too large to send"),
            Error::InvalidEndpoint => write!(f, "Invalid endpoint specified"),
            Error::FailedToMountDevice(e) => {
                write!(f, "Failed to mount device: {}", e)
            }
            Error::NoFreeSlot => write!(f, "No free slot available"),
            Error::Other => write!(f, "Other error occurred"),
        }
    }
}

impl From<dns::StartQueryError> for Error {
    fn from(e: dns::StartQueryError) -> Self {
        match e {
            dns::StartQueryError::InvalidName => Error::InvalidName,
            dns::StartQueryError::NameTooLong => Error::NameTooLong,
            dns::StartQueryError::NoFreeSlot => Error::NoFreeSlot,
        }
    }
}

impl From<dns::GetQueryResultError> for Error {
    fn from(e: dns::GetQueryResultError) -> Self {
        match e {
            dns::GetQueryResultError::Pending => Error::Pending,
            dns::GetQueryResultError::Failed => Error::Failed,
        }
    }
}

impl From<icmp::SendError> for Error {
    fn from(e: icmp::SendError) -> Self {
        match e {
            icmp::SendError::Unaddressable => Error::InvalidEndpoint,
            icmp::SendError::BufferFull => Error::ResourceBusy,
        }
    }
}

impl From<icmp::BindError> for Error {
    fn from(e: icmp::BindError) -> Self {
        match e {
            icmp::BindError::InvalidState => Error::InvalidState,
            icmp::BindError::Unaddressable => Error::NoRoute,
        }
    }
}

impl From<udp::BindError> for Error {
    fn from(e: udp::BindError) -> Self {
        match e {
            udp::BindError::InvalidState => Error::InvalidState,
            udp::BindError::Unaddressable => Error::NoRoute,
        }
    }
}
