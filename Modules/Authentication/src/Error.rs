//! Error handling for the Authentication module.
//!
//! This module defines all possible errors that can occur during authentication operations,
//! including user and group management, file I/O, and password validation.

use core::fmt::Display;

/// Result type alias for authentication operations.
///
/// This is a convenience type that wraps `Result<T, Error_type>` for all
/// authentication-related operations.
pub type Result_type<T> = Result<T, Error_type>;

/// Comprehensive error enumeration for authentication operations.
///
/// This enum covers all possible error conditions that can occur during:
/// - User authentication and management
/// - Group management
/// - File system operations
/// - Password hashing and validation
/// - Random salt generation
#[derive(Debug, Clone)]
pub enum Error_type {
    /// Failed to get the current task identifier
    Failed_to_get_current_task_identifier(Task::Error_type),
    /// Failed to read the users directory from the filesystem
    Failed_to_read_users_directory(File_system::Error_type),
    /// Failed to construct a valid user file path
    Failed_to_get_user_file_path,
    /// Failed to open a user file for reading or writing
    Failed_to_open_user_file(File_system::Error_type),
    /// Failed to read the contents of a user file
    Failed_to_read_user_file(File_system::Error_type),
    /// Failed to parse JSON content from a user file
    Failed_to_parse_user_file(miniserde::Error),
    /// Failed to add a user to the Users manager
    Failed_to_add_user(Users::Error_type),
    /// Failed to generate a new unique user identifier
    Failed_to_get_new_user_identifier(Users::Error_type),
    /// Failed to create a new user account
    Failed_to_create_user(Users::Error_type),
    /// Failed to write user data to a file
    Failed_to_write_user_file(File_system::Error_type),
    /// Failed to create the users directory
    Failed_to_create_users_directory(File_system::Error_type),
    /// Failed to read the groups directory from the filesystem
    Failed_to_read_group_directory(File_system::Error_type),
    /// Failed to construct a valid group file path
    Failed_to_get_group_file_path,
    /// Failed to open a group file for reading or writing
    Failed_to_open_group_file(File_system::Error_type),
    /// Failed to read the contents of a group file
    Failed_to_read_group_file(File_system::Error_type),
    /// Failed to parse JSON content from a group file
    Failed_to_parse_group_file(miniserde::Error),
    /// Failed to add a group to the Users manager
    Failed_to_add_group(Users::Error_type),
    /// Failed to generate a new unique group identifier
    Failed_to_get_new_group_identifier(Users::Error_type),
    /// Failed to create a new group
    Failed_to_create_group(Users::Error_type),
    /// Failed to write group data to a file
    Failed_to_write_group_file(File_system::Error_type),
    /// Failed to create the groups directory
    Failed_to_create_groups_directory(File_system::Error_type),
    /// The provided password is invalid or incorrect
    Invalid_password,
    /// Failed to open the random device for salt generation
    Failed_to_open_random_device(File_system::Error_type),
    /// Failed to read random data from the random device
    Failed_to_read_random_device(File_system::Error_type),
    /// Failed to get user identifier from the Users manager
    Failed_to_get_user_identifier(Users::Error_type),
}

impl Display for Error_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Failed_to_get_current_task_identifier(Error) => {
                write!(
                    Formatter,
                    "Failed to get current task identifier: {}",
                    Error
                )
            }
            Self::Failed_to_read_users_directory(Error) => {
                write!(Formatter, "Failed to read users directory: {}", Error)
            }
            Self::Failed_to_get_user_file_path => {
                write!(Formatter, "Failed to get user file path")
            }
            Self::Failed_to_read_user_file(Error) => {
                write!(Formatter, "Failed to read user file: {}", Error)
            }
            Self::Failed_to_open_user_file(Error) => {
                write!(Formatter, "Failed to open user file: {}", Error)
            }
            Self::Failed_to_parse_user_file(Error) => {
                write!(Formatter, "Failed to parse user file: {}", Error)
            }
            Self::Failed_to_add_user(Error) => {
                write!(Formatter, "Failed to add user: {}", Error)
            }
            Self::Failed_to_create_users_directory(Error) => {
                write!(Formatter, "Failed to create users directory: {}", Error)
            }
            Self::Failed_to_read_group_directory(Error) => {
                write!(Formatter, "Failed to read group directory: {}", Error)
            }
            Self::Failed_to_get_group_file_path => {
                write!(Formatter, "Failed to get group file path")
            }
            Self::Failed_to_open_group_file(Error) => {
                write!(Formatter, "Failed to open group file: {}", Error)
            }
            Self::Failed_to_read_group_file(Error) => {
                write!(Formatter, "Failed to read group file: {}", Error)
            }
            Self::Failed_to_parse_group_file(Error) => {
                write!(Formatter, "Failed to parse group file: {}", Error)
            }
            Self::Failed_to_add_group(Error) => {
                write!(Formatter, "Failed to add group: {}", Error)
            }
            Self::Failed_to_create_groups_directory(Error) => {
                write!(Formatter, "Failed to create group directory: {}", Error)
            }
            Self::Invalid_password => {
                write!(Formatter, "Invalid password")
            }
            Self::Failed_to_open_random_device(Error) => {
                write!(Formatter, "Failed to open random device: {}", Error)
            }
            Self::Failed_to_read_random_device(Error) => {
                write!(Formatter, "Failed to read random device: {}", Error)
            }
            Self::Failed_to_create_user(Error) => {
                write!(Formatter, "Failed to create user: {}", Error)
            }
            Self::Failed_to_get_new_user_identifier(Error) => {
                write!(Formatter, "Failed to get new user identifier: {}", Error)
            }
            Self::Failed_to_write_user_file(Error) => {
                write!(Formatter, "Failed to write user file: {}", Error)
            }
            Self::Failed_to_get_new_group_identifier(Error) => {
                write!(Formatter, "Failed to get new group identifier: {}", Error)
            }
            Self::Failed_to_create_group(Error) => {
                write!(Formatter, "Failed to create group: {}", Error)
            }
            Self::Failed_to_write_group_file(Error) => {
                write!(Formatter, "Failed to write group file: {}", Error)
            }
            Self::Failed_to_get_user_identifier(Error) => {
                write!(Formatter, "Failed to get user identifier: {}", Error)
            }
        }
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(Error: Task::Error_type) -> Self {
        Self::Failed_to_get_current_task_identifier(Error)
    }
}
