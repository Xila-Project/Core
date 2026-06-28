use core::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidArgument(getargs_derive::Error),
    WasmError(wasmi::Error),
    LinkError(wasmi::errors::LinkerError),
}

impl From<wasmi::Error> for Error {
    fn from(error: wasmi::Error) -> Self {
        Self::WasmError(error)
    }
}

impl From<wasmi::errors::LinkerError> for Error {
    fn from(error: wasmi::errors::LinkerError) -> Self {
        Self::LinkError(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("An error occurred")
    }
}

impl core::error::Error for Error {}
