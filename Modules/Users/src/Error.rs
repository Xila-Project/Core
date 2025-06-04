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
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Duplicate_group_identifier => {
                write!(Formatter, "Duplicate group identifier")
            }
            Self::Duplicate_group_name => {
                write!(Formatter, "Duplicate group name")
            }
            Self::Duplicate_user_identifier => {
                write!(Formatter, "Duplicate user identifier")
            }
            Self::Duplicate_user_name => {
                write!(Formatter, "Duplicate user name")
            }
            Self::Invalid_group_identifier => {
                write!(Formatter, "Invalid group identifier")
            }
            Self::Invalid_user_identifier => {
                write!(Formatter, "Invalid user identifier")
            }
            Self::Too_many_groups => {
                write!(Formatter, "Too many groups")
            }
            Self::Too_many_users => {
                write!(Formatter, "Too many users")
            }
            Self::Poisoned_lock => {
                write!(Formatter, "Poisoned lock")
            }
            Self::Not_initialized => {
                write!(Formatter, "Not initialized")
            }
            Self::Already_initialized => {
                write!(Formatter, "Already initialized")
            }
        }
    }
}
