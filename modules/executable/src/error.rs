use core::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    FileSystem(file_system::Error),
    Task(task::Error),
    FailedToGetMainFunction,
    InvalidStackSize,
    PermissionDenied,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::FileSystem(error) => write!(formatter, "{error}"),
            Error::Task(error) => write!(formatter, "{error}"),
            Error::FailedToGetMainFunction => {
                write!(formatter, "Failed to get main function")
            }
            Error::InvalidStackSize => write!(formatter, "Invalid stack size"),
            Error::PermissionDenied => write!(formatter, "Permission denied"),
        }
    }
}

impl From<file_system::Error> for Error {
    fn from(error: file_system::Error) -> Self {
        Error::FileSystem(error)
    }
}

impl From<task::Error> for Error {
    fn from(error: task::Error) -> Self {
        Error::Task(error)
    }
}
