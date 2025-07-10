use std::io;

pub fn map_error(error: io::Error) -> file_system::Error {
    match error.kind() {
        io::ErrorKind::PermissionDenied => file_system::Error::PermissionDenied,
        io::ErrorKind::NotFound => file_system::Error::NotFound,
        io::ErrorKind::AlreadyExists => file_system::Error::AlreadyExists,
        io::ErrorKind::InvalidInput => file_system::Error::InvalidPath,
        io::ErrorKind::InvalidData => file_system::Error::InvalidFile,
        _ => file_system::Error::Unknown,
    }
}
