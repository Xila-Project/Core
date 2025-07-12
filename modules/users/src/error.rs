use core::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Error {
    DuplicateGroupIdentifier,
    DuplicateGroupName,
    DuplicateUserIdentifier,
    DuplicateUserName,
    InvalidGroupIdentifier,
    InvalidUserIdentifier,
    TooManyGroups,
    TooManyUsers,
    PoisonedLock,
    NotInitialized,
    AlreadyInitialized,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::DuplicateGroupIdentifier => {
                write!(formatter, "Duplicate group identifier")
            }
            Self::DuplicateGroupName => {
                write!(formatter, "Duplicate group name")
            }
            Self::DuplicateUserIdentifier => {
                write!(formatter, "Duplicate user identifier")
            }
            Self::DuplicateUserName => {
                write!(formatter, "Duplicate user name")
            }
            Self::InvalidGroupIdentifier => {
                write!(formatter, "Invalid group identifier")
            }
            Self::InvalidUserIdentifier => {
                write!(formatter, "Invalid user identifier")
            }
            Self::TooManyGroups => {
                write!(formatter, "Too many groups")
            }
            Self::TooManyUsers => {
                write!(formatter, "Too many users")
            }
            Self::PoisonedLock => {
                write!(formatter, "Poisoned lock")
            }
            Self::NotInitialized => {
                write!(formatter, "Not initialized")
            }
            Self::AlreadyInitialized => {
                write!(formatter, "Already initialized")
            }
        }
    }
}
