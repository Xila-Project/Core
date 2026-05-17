#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidArgumentsCount,
    InvalidPointer,
    EnvironmentRetrievalFailed,
}

pub type Result<T> = core::result::Result<T, Error>;
