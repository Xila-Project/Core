#![allow(non_camel_case_types)]

#[derive(Debug)]
pub enum Error_type {
    Invalid_pointer,
    Invalid_length,
    Invalid_UTF8_string,
    Buffer_too_small,
    Failed_to_convert_length_to_S,
}
