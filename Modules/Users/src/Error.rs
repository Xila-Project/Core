use std::sync::PoisonError;

pub type Result<T> = std::result::Result<T, Error_type>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Error_type {
    Duplicate_group_identifier,
    Duplicate_user_identifier,
    Invalid_group_identifier,
    Invalid_user_identifier,
    Too_many_groups,
    Too_many_users,
    Poisoned_lock,
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Error_type::Poisoned_lock
    }
}
