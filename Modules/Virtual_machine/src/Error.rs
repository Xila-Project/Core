#![allow(non_camel_case_types)]

#[derive(Debug)]
pub enum Error_type {
    Invalid_pointer,
    Invalid_UTF8_string,
    Slice_conversion_failed(Shared::Error_type),
}
