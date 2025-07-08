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
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Failed_to_get_current_task_identifier(error) => {
                write!(formatter, "Failed to get current task identifier: {error}")
            }
            Self::Failed_to_read_users_directory(error) => {
                write!(formatter, "Failed to read users directory: {error}")
            }
            Self::Failed_to_get_user_file_path => {
                write!(formatter, "Failed to get user file path")
            }
            Self::Failed_to_read_user_file(error) => {
                write!(formatter, "Failed to read user file: {error}")
            }
            Self::Failed_to_open_user_file(error) => {
                write!(formatter, "Failed to open user file: {error}")
            }
            Self::Failed_to_parse_user_file(error) => {
                write!(formatter, "Failed to parse user file: {error}")
            }
            Self::Failed_to_add_user(error) => {
                write!(formatter, "Failed to add user: {error}")
            }
            Self::Failed_to_create_users_directory(error) => {
                write!(formatter, "Failed to create users directory: {error}")
            }
            Self::Failed_to_read_group_directory(error) => {
                write!(formatter, "Failed to read group directory: {error}")
            }
            Self::Failed_to_get_group_file_path => {
                write!(formatter, "Failed to get group file path")
            }
            Self::Failed_to_open_group_file(error) => {
                write!(formatter, "Failed to open group file: {error}")
            }
            Self::Failed_to_read_group_file(error) => {
                write!(formatter, "Failed to read group file: {error}")
            }
            Self::Failed_to_parse_group_file(error) => {
                write!(formatter, "Failed to parse group file: {error}")
            }
            Self::Failed_to_add_group(error) => {
                write!(formatter, "Failed to add group: {error}")
            }
            Self::Failed_to_create_groups_directory(error) => {
                write!(formatter, "Failed to create group directory: {error}")
            }
            Self::Invalid_password => {
                write!(formatter, "Invalid password")
            }
            Self::Failed_to_open_random_device(error) => {
                write!(formatter, "Failed to open random device: {error}")
            }
            Self::Failed_to_read_random_device(error) => {
                write!(formatter, "Failed to read random device: {error}")
            }
            Self::Failed_to_create_user(error) => {
                write!(formatter, "Failed to create user: {error}")
            }
            Self::Failed_to_get_new_user_identifier(error) => {
                write!(formatter, "Failed to get new user identifier: {error}")
            }
            Self::Failed_to_write_user_file(error) => {
                write!(formatter, "Failed to write user file: {error}")
            }
            Self::Failed_to_get_new_group_identifier(error) => {
                write!(formatter, "Failed to get new groupe identifier: {error}")
            }
            Self::Failed_to_create_group(error) => {
                write!(formatter, "Failed to create group: {error}")
            }
            Self::Failed_to_write_group_file(error) => {
                write!(
                    formatter,
                    "Failed to writeerror
                 group file: {error}"
                )
            }
            Self::Failed_to_get_user_identifier(error) => {
                write!(formatter, "Failed to get user identifier: {error}")
            }
        }
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(error: Task::Error_type) -> Self {
        Self::Failed_to_get_current_task_identifier(error)
    }
}
