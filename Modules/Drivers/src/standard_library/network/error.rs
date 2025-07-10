use std::io::{self, ErrorKind};

use network::Error;

pub fn into_socket_error(error: io::Error) -> Error {
    match error.kind() {
        ErrorKind::NotFound => Error::NotFound,
        ErrorKind::PermissionDenied => Error::PermissionDenied,
        ErrorKind::ConnectionRefused => Error::ConnectionRefused,
        ErrorKind::ConnectionReset => Error::ConnectionReset,
        ErrorKind::ConnectionAborted => Error::ConnectionAborted,
        ErrorKind::HostUnreachable => Error::HostUnreachable,
        ErrorKind::NetworkUnreachable => Error::NetworkUnreachable,
        ErrorKind::NotConnected => Error::NotConnected,
        ErrorKind::AddrInUse => Error::AddressInUse,
        ErrorKind::AddrNotAvailable => Error::AddressNotAvailable,
        ErrorKind::NetworkDown => Error::NetworkDown,
        ErrorKind::BrokenPipe => Error::BrokenPipe,
        ErrorKind::AlreadyExists => Error::AlreadyExists,
        ErrorKind::WouldBlock => Error::WouldBlock,

        // ErrorKind::FilesystemLoop => Error::Filesystem_loop,
        ErrorKind::InvalidInput => Error::InvalidInput,
        ErrorKind::InvalidData => Error::InvalidData,
        ErrorKind::TimedOut => Error::TimedOut,
        ErrorKind::WriteZero => Error::WriteZero,
        ErrorKind::StorageFull => Error::StorageFull,

        // ErrorKind::FilesystemQuotaExceeded => Error::Filesystem_quota_exceeded,
        ErrorKind::ResourceBusy => Error::ResourceBusy,

        ErrorKind::Deadlock => Error::Deadlock,
        // ErrorKind::CrossesDevices => todo!(),

        // ErrorKind::InvalidFilename => todo!(),
        ErrorKind::ArgumentListTooLong => todo!(),
        ErrorKind::Interrupted => Error::Interrupted,
        ErrorKind::Unsupported => Error::Unsupported,
        ErrorKind::UnexpectedEof => Error::UnexpectedEndOfFile,
        ErrorKind::OutOfMemory => Error::OutOfMemory,
        // ErrorKind::InProgress => todo!(),
        ErrorKind::Other => Error::Other,
        _ => todo!(),
    }
}
