use std::sync::PoisonError;

pub type Result<T> = std::result::Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(C)]
pub enum Error_type {
    Invalid_task_identifier,
    Failed_to_create_thread,
    No_thread_for_task,
    Failed_to_spawn_thread,
    Poisoned_lock,
    Invalid_environment_variable,
    Too_many_tasks,
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Error_type::Poisoned_lock
    }
}
