use std::{num::NonZeroU32, sync::PoisonError};

pub type Result_type<T> = std::result::Result<T, Error_type>;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub enum Error_type {
    Failed_to_initialize_file_system = 1,
    Permission_denied,
    Not_found,
    Already_exists,
    Directory_already_exists,
    File_system_full,
    File_system_error,
    Invalid_path,
    Invalid_file,
    Invalid_directory,
    Invalid_symbolic_link,
    Unknown,
    Invalid_identifier,
    Failed_to_get_task_informations,
    Too_many_mounted_file_systems,
    Poisoned_lock,
    Too_many_open_files,
    Internal_error,
    Invalid_mode,
    Unsupported_operation,
    Ressource_busy,
    Already_initialized,
    Not_initialized,
    Failed_to_get_users_manager_instance,
    Failed_to_get_task_manager_instance,
    Invalid_input,
    Other,
}

#[cfg(feature = "std")]
impl From<std::io::ErrorKind> for Error_type {
    fn from(Error: std::io::ErrorKind) -> Self {
        use std::io::ErrorKind;

        match Error {
            ErrorKind::PermissionDenied => Error_type::Permission_denied,
            ErrorKind::NotFound => Error_type::Not_found,
            ErrorKind::AlreadyExists => Error_type::Already_exists,
            ErrorKind::InvalidInput => Error_type::Invalid_path,
            ErrorKind::InvalidData => Error_type::Invalid_file,
            _ => Error_type::Unknown,
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error_type {
    fn from(Error: std::io::Error) -> Self {
        Error.kind().into()
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(_: Task::Error_type) -> Self {
        Error_type::Failed_to_get_task_informations
    }
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Error_type::Poisoned_lock
    }
}

impl From<Error_type> for NonZeroU32 {
    fn from(Error: Error_type) -> Self {
        unsafe { NonZeroU32::new_unchecked(Error as u32) }
    }
}
