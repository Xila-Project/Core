use std::io::{self, ErrorKind};

use Network::Error_type;

pub fn Into_socket_error(error: io::Error) -> Error_type {
    match error.kind() {
        ErrorKind::NotFound => Error_type::Not_found,
        ErrorKind::PermissionDenied => Error_type::Permission_denied,
        ErrorKind::ConnectionRefused => Error_type::Connection_refused,
        ErrorKind::ConnectionReset => Error_type::Connection_reset,
        ErrorKind::ConnectionAborted => Error_type::Connection_aborted,
        ErrorKind::HostUnreachable => Error_type::Host_unreachable,
        ErrorKind::NetworkUnreachable => Error_type::Network_unreachable,
        ErrorKind::NotConnected => Error_type::Not_connected,
        ErrorKind::AddrInUse => Error_type::Address_in_use,
        ErrorKind::AddrNotAvailable => Error_type::Address_not_available,
        ErrorKind::NetworkDown => Error_type::Network_down,
        ErrorKind::BrokenPipe => Error_type::Broken_pipe,
        ErrorKind::AlreadyExists => Error_type::Already_exists,
        ErrorKind::WouldBlock => Error_type::Would_block,

        // ErrorKind::FilesystemLoop => Error_type::Filesystem_loop,
        ErrorKind::InvalidInput => Error_type::Invalid_input,
        ErrorKind::InvalidData => Error_type::Invalid_data,
        ErrorKind::TimedOut => Error_type::Timed_out,
        ErrorKind::WriteZero => Error_type::Write_zero,
        ErrorKind::StorageFull => Error_type::Storage_full,

        // ErrorKind::FilesystemQuotaExceeded => Error_type::Filesystem_quota_exceeded,
        ErrorKind::ResourceBusy => Error_type::Resource_busy,

        ErrorKind::Deadlock => Error_type::Deadlock,
        // ErrorKind::CrossesDevices => todo!(),

        // ErrorKind::InvalidFilename => todo!(),
        ErrorKind::ArgumentListTooLong => todo!(),
        ErrorKind::Interrupted => Error_type::Interrupted,
        ErrorKind::Unsupported => Error_type::Unsupported,
        ErrorKind::UnexpectedEof => Error_type::Unexpected_end_of_file,
        ErrorKind::OutOfMemory => Error_type::Out_of_memory,
        // ErrorKind::InProgress => todo!(),
        ErrorKind::Other => Error_type::Other,
        _ => todo!(),
    }
}
