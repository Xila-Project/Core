use core::fmt::Display;

pub type Result_type<T> = core::result::Result<T, Error_type>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Error_type {
    Duplicate_group_identifier,
    Duplicate_group_name,
    Duplicate_user_identifier,
    Duplicate_user_name,
    Invalid_group_identifier,
    Invalid_user_identifier,
    Too_many_groups,
    Too_many_users,
    Poisoned_lock,
    Not_initialized,
    Already_initialized,
}

impl Display for Error_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Duplicate_group_identifier => {
                write!(formatter, "Duplicate group identifier")
            }
            Self::Duplicate_group_name => {
                write!(formatter, "Duplicate group name")
            }
            Self::Duplicate_user_identifier => {
                write!(formatter, "Duplicate user identifier")
            }
            Self::Duplicate_user_name => {
                write!(formatter, "Duplicate user name")
            }
            Self::Invalid_group_identifier => {
                write!(formatter, "Invalid group identifier")
            }
            Self::Invalid_user_identifier => {
                write!(formatter, "Invalid user identifier")
            }
            Self::Too_many_groups => {
                write!(formatter, "Too many groups")
            }
            Self::Too_many_users => {
                write!(formatter, "Too many users")
            }
            Self::Poisoned_lock => {
                write!(formatter, "Poisoned lock")
            }
            Self::Not_initialized => {
                write!(formatter, "Not initialized")
            }
            Self::Already_initialized => {
                write!(formatter, "Already initialized")
            }
        }
    }
}
