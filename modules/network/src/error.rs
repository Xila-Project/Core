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
            Error::InProgress => write!(f, "In progress operation not completed yet"),
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
            Error::Failed => write!(f, "Name lookup failed"),
            Error::InvalidState => write!(f, "Invalid state for operation"),
            Error::InvalidPort => write!(f, "Invalid port specified"),
            Error::NoRoute => write!(f, "No route to host"),
            Error::Truncated => write!(f, "Truncated packet received"),
            Error::SocketNotBound => write!(f, "Socket not bound"),
            Error::PacketTooLarge => write!(f, "Packet too large to send"),
            Error::Other => write!(f, "Other error occurred"),
        }
    }
}

impl From<embassy_net::dns::Error> for Error {
    fn from(e: embassy_net::dns::Error) -> Self {
        match e {
            embassy_net::dns::Error::InvalidName => Error::InvalidName,
            embassy_net::dns::Error::NameTooLong => Error::NameTooLong,
            embassy_net::dns::Error::Failed => Error::Failed,
        }
    }
}

impl From<embassy_net::tcp::AcceptError> for Error {
    fn from(e: embassy_net::tcp::AcceptError) -> Self {
        match e {
            embassy_net::tcp::AcceptError::InvalidState => Error::InvalidState,
            embassy_net::tcp::AcceptError::InvalidPort => Error::InvalidPort,
            embassy_net::tcp::AcceptError::ConnectionReset => Error::ConnectionReset,
        }
    }
}

impl From<embassy_net::tcp::ConnectError> for Error {
    fn from(e: embassy_net::tcp::ConnectError) -> Self {
        match e {
            embassy_net::tcp::ConnectError::InvalidState => Error::InvalidState,
            embassy_net::tcp::ConnectError::ConnectionReset => Error::ConnectionReset,
            embassy_net::tcp::ConnectError::TimedOut => Error::TimedOut,
            embassy_net::tcp::ConnectError::NoRoute => Error::HostUnreachable,
        }
    }
}

impl From<embassy_net::tcp::Error> for Error {
    fn from(e: embassy_net::tcp::Error) -> Self {
        match e {
            embassy_net::tcp::Error::ConnectionReset => Error::ConnectionReset,
        }
    }
}

impl From<embassy_net::udp::RecvError> for Error {
    fn from(e: embassy_net::udp::RecvError) -> Self {
        match e {
            embassy_net::udp::RecvError::Truncated => Error::Truncated,
        }
    }
}

impl From<embassy_net::udp::SendError> for Error {
    fn from(e: embassy_net::udp::SendError) -> Self {
        match e {
            embassy_net::udp::SendError::SocketNotBound => Error::SocketNotBound,
            embassy_net::udp::SendError::PacketTooLarge => Error::PacketTooLarge,
            embassy_net::udp::SendError::NoRoute => Error::NoRoute,
        }
    }
}

impl From<embassy_net::udp::BindError> for Error {
    fn from(e: embassy_net::udp::BindError) -> Self {
        match e {
            embassy_net::udp::BindError::InvalidState => Error::InvalidState,
            embassy_net::udp::BindError::NoRoute => Error::NoRoute,
        }
    }
}
