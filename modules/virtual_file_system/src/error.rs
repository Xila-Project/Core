use core::{fmt::Display, num::NonZeroU32};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    UnavailableDriver,
    InvalidFileSystem,
    InvalidParameter,
    TooManyOpenFiles,
    FailedToGetTaskInformations,
    FileSystem(file_system::Error) = 0xFF,
    Network(network::Error) = 0x200,
}

impl Error {
    pub fn get_discriminant(&self) -> NonZeroU32 {
        unsafe { *<*const _>::from(self).cast::<NonZeroU32>() }
    }
}

impl From<Error> for NonZeroU32 {
    fn from(value: Error) -> Self {
        let discriminant = value.get_discriminant();

        let offset = match value {
            Error::FileSystem(error_type) => error_type.get_discriminant().get(),
            Error::Network(error_type) => error_type.get_discriminant().get() as u32,
            _ => 0,
        };

        discriminant.saturating_add(offset)
    }
}

impl From<file_system::Error> for Error {
    fn from(value: file_system::Error) -> Self {
        Self::FileSystem(value)
    }
}

impl From<network::Error> for Error {
    fn from(value: network::Error) -> Self {
        Self::Network(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::AlreadyInitialized => write!(f, "Already initialized"),
            Error::UnavailableDriver => write!(f, "Unavailable driver"),
            Error::InvalidFileSystem => write!(f, "Invalid file system"),
            Error::InvalidParameter => write!(f, "Invalid parameter"),
            Error::TooManyOpenFiles => write!(f, "Too many open files"),
            Error::FailedToGetTaskInformations => {
                write!(f, "Failed to get task informations")
            }
            Error::FileSystem(err) => write!(f, "File system error: {err}"),
            Error::Network(err) => write!(f, "Network error: {err}"),
        }
    }
}
