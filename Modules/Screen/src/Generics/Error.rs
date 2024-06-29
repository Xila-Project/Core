#![allow(non_camel_case_types)]

pub type Result_type<T> = std::result::Result<T, Error_type>;

#[derive(Debug, Clone)]
pub enum Error_type {
    Invalid_dimension,
    Unknown(String),
}
