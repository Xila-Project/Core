use core::{fmt::Display, num::NonZeroU32};

use embedded_io_async::ErrorKind;

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
    InvalidIdentifier,
    AlreadyExists,
    Time(time::Error),
    FileSystem(file_system::Error) = 0x100,
    Users(users::Error) = 0x300,
    Task(task::Error) = 0x400,
    MissingAttribute,
    InvalidPath,
    PermissionDenied,
    TooManyInodes,
    RessourceBusy,
    NotADirectory,
    NotAFile,
    InvalidInode,
    InvalidMode,
    InvalidOpen,
    UnsupportedOperation,
    FailedToWrite,
    DelimiterNotFound,
    Orphaned,
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
            _ => 0,
        };

        discriminant.saturating_add(offset)
    }
}

impl From<users::Error> for Error {
    fn from(value: users::Error) -> Self {
        Self::Users(value)
    }
}

impl From<time::Error> for Error {
    fn from(value: time::Error) -> Self {
        Self::Time(value)
    }
}

impl From<file_system::Error> for Error {
    fn from(value: file_system::Error) -> Self {
        Self::FileSystem(value)
    }
}

impl From<task::Error> for Error {
    fn from(value: task::Error) -> Self {
        Self::Task(value)
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
            Error::InvalidIdentifier => write!(f, "Invalid identifier"),
            Error::AlreadyExists => write!(f, "Already exists"),
            Error::Time(err) => write!(f, "Time error: {err}"),
            Error::Users(err) => write!(f, "Users error: {err}"),
            Error::Task(err) => write!(f, "Task error: {err}"),
            Error::MissingAttribute => write!(f, "Missing attribute"),
            Error::InvalidPath => write!(f, "Invalid path"),
            Error::PermissionDenied => write!(f, "Permission denied"),
            Error::TooManyInodes => write!(f, "Too many inodes"),
            Error::RessourceBusy => write!(f, "Ressource busy"),
            Error::NotADirectory => write!(f, "Not a directory"),
            Error::NotAFile => write!(f, "Not a file"),
            Error::InvalidMode => write!(f, "Invalid mode"),
            Error::InvalidOpen => write!(f, "Invalid open"),
            Error::InvalidInode => write!(f, "Invalid inode"),
            Error::UnsupportedOperation => write!(f, "Unsupported operation"),
            Error::FailedToWrite => write!(f, "Failed to write"),
            Error::DelimiterNotFound => write!(f, "Delimiter not found"),
            Error::Orphaned => write!(f, "Orphaned"),
        }
    }
}

impl core::error::Error for Error {}

impl embedded_io_async::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Error::PermissionDenied => ErrorKind::PermissionDenied,
            Error::NotADirectory => ErrorKind::InvalidInput,
            Error::NotAFile => ErrorKind::InvalidInput,
            Error::InvalidMode => ErrorKind::InvalidInput,
            Error::InvalidOpen => ErrorKind::InvalidInput,
            Error::UnsupportedOperation => ErrorKind::Unsupported,
            _ => ErrorKind::Other,
        }
    }
}
