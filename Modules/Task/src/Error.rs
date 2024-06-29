#![allow(non_camel_case_types)]

use std::{num::NonZeroU32, sync::PoisonError};

pub type Result_type<T> = std::result::Result<T, Error_type>;

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

impl From<Error_type> for NonZeroU32 {
    fn from(Error: Error_type) -> Self {
        unsafe { NonZeroU32::new_unchecked(Error as u32) }
    }
}
