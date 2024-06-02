use wamr_rust_sdk::RuntimeError;

#[derive(Debug)]
pub enum Error_type {
    Invalid_pointer,
    Invalid_UTF8_string,
    Slice_conversion_failed(Shared::Error_type),
}
