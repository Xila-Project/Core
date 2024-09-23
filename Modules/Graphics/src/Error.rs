use std::sync::PoisonError;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Error_type {
    Invalid_reference,
    Already_initialized,
    Failed_to_create_thread,
    Not_initialized,
    Out_of_memory,
    Already_in_use,
    Poisoned_lock,
    Failed_to_register,
    Failed_to_get_resolution,
    Not_registered,
    Not_available,
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Error_type::Poisoned_lock
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(_: Task::Error_type) -> Self {
        Error_type::Failed_to_create_thread
    }
}
