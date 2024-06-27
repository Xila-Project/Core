use std::{num::NonZeroU32, sync::PoisonError};

use Shared::Error_discriminant_trait;

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
    Other,
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
impl Error_discriminant_trait for Error_type {
    fn Get_discriminant(&self) -> NonZeroU32 {
        NonZeroU32::new(*self as u32).unwrap()
    }

    fn From_discriminant(Discriminant: NonZeroU32) -> Self {
        unsafe { std::mem::transmute(Discriminant.get()) }
    }
}
