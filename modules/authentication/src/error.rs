//! Error handling for the Authentication module.
//!
//! This module defines all possible errors that can occur during authentication operations,
//! including user and group management, file I/O, and password validation.

use core::fmt::Display;

/// Result type alias for authentication operations.
///
/// This is a convenience type that wraps `Result<T, Error>` for all
/// authentication-related operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Comprehensive error enumeration for authentication operations.
///
/// This enum covers all possible error conditions that can occur during:
/// - User authentication and management
/// - Group management
/// - File system operations
/// - Password hashing and validation
/// - Random salt generation
#[derive(Debug, Clone)]
pub enum Error {
    /// Failed to get the current task identifier
    FailedToGetCurrentTaskIdentifier(task::Error),
    /// Failed to read the users directory from the filesystem
    FailedToReadUsersDirectory(file_system::Error),
    /// Failed to construct a valid user file path
    FailedToGetUserFilePath,
    /// Failed to open a user file for reading or writing
    FailedToOpenUserFile(file_system::Error),
    /// Failed to read the contents of a user file
    FailedToReadUserFile(file_system::Error),
    /// Failed to parse JSON content from a user file
    FailedToParseUserFile(miniserde::Error),
    /// Failed to add a user to the Users manager
    FailedToAddUser(users::Error),
    /// Failed to generate a new unique user identifier
    FailedToGetNewUserIdentifier(users::Error),
    /// Failed to create a new user account
    FailedToCreateUser(users::Error),
    /// Failed to write user data to a file
    FailedToWriteUserFile(file_system::Error),
    /// Failed to create the users directory
    FailedToCreateUsersDirectory(file_system::Error),
    /// Failed to read the groups directory from the filesystem
    FailedToReadGroupDirectory(file_system::Error),
    /// Failed to construct a valid group file path
    FailedToGetGroupFilePath,
    /// Failed to open a group file for reading or writing
    FailedToOpenGroupFile(file_system::Error),
    /// Failed to read the contents of a group file
    FailedToReadGroupFile(file_system::Error),
    /// Failed to parse JSON content from a group file
    FailedToParseGroupFile(miniserde::Error),
    /// Failed to add a group to the Users manager
    FailedToAddGroup(users::Error),
    /// Failed to generate a new unique group identifier
    FailedToGetNewGroupIdentifier(users::Error),
    /// Failed to create a new group
    FailedToCreateGroup(users::Error),
    /// Failed to write group data to a file
    FailedToWriteGroupFile(file_system::Error),
    /// Failed to create the groups directory
    FailedToCreateGroupsDirectory(file_system::Error),
    /// The provided password is invalid or incorrect
    InvalidPassword,
    /// Failed to open the random device for salt generation
    FailedToOpenRandomDevice(file_system::Error),
    /// Failed to read random data from the random device
    FailedToReadRandomDevice(file_system::Error),
    /// Failed to get user identifier from the Users manager
    FailedToGetUserIdentifier(users::Error),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::FailedToGetCurrentTaskIdentifier(error) => {
                write!(formatter, "Failed to get current task identifier: {error}")
            }
            Self::FailedToReadUsersDirectory(error) => {
                write!(formatter, "Failed to read users directory: {error}")
            }
            Self::FailedToGetUserFilePath => {
                write!(formatter, "Failed to get user file path")
            }
            Self::FailedToReadUserFile(error) => {
                write!(formatter, "Failed to read user file: {error}")
            }
            Self::FailedToOpenUserFile(error) => {
                write!(formatter, "Failed to open user file: {error}")
            }
            Self::FailedToParseUserFile(error) => {
                write!(formatter, "Failed to parse user file: {error}")
            }
            Self::FailedToAddUser(error) => {
                write!(formatter, "Failed to add user: {error}")
            }
            Self::FailedToCreateUsersDirectory(error) => {
                write!(formatter, "Failed to create users directory: {error}")
            }
            Self::FailedToReadGroupDirectory(error) => {
                write!(formatter, "Failed to read group directory: {error}")
            }
            Self::FailedToGetGroupFilePath => {
                write!(formatter, "Failed to get group file path")
            }
            Self::FailedToOpenGroupFile(error) => {
                write!(formatter, "Failed to open group file: {error}")
            }
            Self::FailedToReadGroupFile(error) => {
                write!(formatter, "Failed to read group file: {error}")
            }
            Self::FailedToParseGroupFile(error) => {
                write!(formatter, "Failed to parse group file: {error}")
            }
            Self::FailedToAddGroup(error) => {
                write!(formatter, "Failed to add group: {error}")
            }
            Self::FailedToCreateGroupsDirectory(error) => {
                write!(formatter, "Failed to create group directory: {error}")
            }
            Self::InvalidPassword => {
                write!(formatter, "Invalid password")
            }
            Self::FailedToOpenRandomDevice(error) => {
                write!(formatter, "Failed to open random device: {error}")
            }
            Self::FailedToReadRandomDevice(error) => {
                write!(formatter, "Failed to read random device: {error}")
            }
            Self::FailedToCreateUser(error) => {
                write!(formatter, "Failed to create user: {error}")
            }
            Self::FailedToGetNewUserIdentifier(error) => {
                write!(formatter, "Failed to get new user identifier: {error}")
            }
            Self::FailedToWriteUserFile(error) => {
                write!(formatter, "Failed to write user file: {error}")
            }
            Self::FailedToGetNewGroupIdentifier(error) => {
                write!(formatter, "Failed to get new groupe identifier: {error}")
            }
            Self::FailedToCreateGroup(error) => {
                write!(formatter, "Failed to create group: {error}")
            }
            Self::FailedToWriteGroupFile(error) => {
                write!(
                    formatter,
                    "Failed to writeerror
                 group file: {error}"
                )
            }
            Self::FailedToGetUserIdentifier(error) => {
                write!(formatter, "Failed to get user identifier: {error}")
            }
        }
    }
}

impl From<task::Error> for Error {
    fn from(error: task::Error) -> Self {
        Self::FailedToGetCurrentTaskIdentifier(error)
    }
}
