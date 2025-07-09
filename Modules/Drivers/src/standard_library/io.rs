use std::{io::Error, io::ErrorKind};

use file_system::Error_type;

pub fn map_error(Error: Error) -> Error_type {
    match Error.kind() {
        ErrorKind::PermissionDenied => Error_type::Permission_denied,
        ErrorKind::NotFound => Error_type::Not_found,
        ErrorKind::AlreadyExists => Error_type::Already_exists,
        ErrorKind::InvalidInput => Error_type::Invalid_path,
        ErrorKind::InvalidData => Error_type::Invalid_file,
        _ => Error_type::Unknown,
    }
}
