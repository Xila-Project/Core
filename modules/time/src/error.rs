#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NotInitialized,
    AlreadyInitialized,
    DeviceError(file_system::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
