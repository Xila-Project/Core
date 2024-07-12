#![allow(non_camel_case_types)]

use std::sync::PoisonError;

pub type Result_type<T> = std::result::Result<T, Error_type>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error_type {
    Invalid_dimension,
    Poisoned_lock,
    Unknown(String),
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Error_type::Poisoned_lock
    }
}
