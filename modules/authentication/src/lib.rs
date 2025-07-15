//! # Authentication Module
//!
//! The Authentication module provides user and group management functionality for the Xila operating system.
//! It handles user authentication, password hashing, and secure storage of user/group information.
//!
//! ## Features
//!
//! - **User Management**: Create, authenticate, and manage user accounts
//! - **Group Management**: Create and manage user groups
//! - **Password Security**: Secure password hashing using SHA-512 with salt
//! - **File-based Storage**: Persistent storage of user and group data in JSON format
//! - **Async Operations**: All operations are asynchronous for better performance
//!
//! ## Usage
//!
//! The module provides functionality to:
//! - Load existing users and groups from the filesystem
//! - Authenticate users with username/password
//! - Create new users and groups
//! - Change user passwords and usernames
//! - Hash passwords securely with salt generation
//!
//! ## File Structure
//!
//! - Users are stored in `/System/Users/` directory
//! - Groups are stored in `/System/Groups/` directory
//! - Each user/group has their own JSON file containing their data
//! - Random salt generation uses `/devices/random` device

#![no_std]

extern crate alloc;

mod error;
mod group;
mod hash;
mod user;

use alloc::{vec, vec::Vec};
pub use error::*;
pub use group::*;
pub use user::*;

/// Path to the users directory in the filesystem
const USERS_FOLDER_PATH: &str = "/system/users";

/// Path to the groups directory in the filesystem
const GROUP_FOLDER_PATH: &str = "/system/groups";

/// Path to the random device used for salt generation
const RANDOM_DEVICE_PATH: &str = "/devices/random";

/// Loads all users and groups from the filesystem into memory.
///
/// This function scans the `/System/Users` and `/System/Groups` directories,
/// reads all user and group files, and adds them to the Users manager.
/// It should be called during system initialization.
///
/// # Returns
///
/// Returns `Ok(())` if all users and groups were loaded successfully,
/// or an `Error` if any operation failed.
///
/// # Errors
///
/// This function can return errors in the following cases:
/// - Failed to read users or groups directory
/// - Failed to parse user or group files
/// - Failed to add users or groups to the Users manager
///
/// # Examples
///
/// ```rust
/// use authentication::load_all_users_and_groups;
///
/// async fn example() -> Result<(), authentication::Error> {
///     load_all_users_and_groups().await?;
///     Ok(())
/// }
/// ```
pub async fn load_all_users_and_groups() -> Result<()> {
    use group::read_group_file;
    use user::read_user_file;
    use virtual_file_system::Directory;
    // Open Xila users folder.
    let virtual_file_system = virtual_file_system::get_instance();

    let users_manager = users::get_instance();

    let mut buffer: Vec<u8> = vec![];

    {
        let groups_directory = Directory::open(virtual_file_system, GROUP_FOLDER_PATH)
            .await
            .map_err(Error::FailedToReadGroupDirectory)?;

        // Read all groups.
        for group_entry in groups_directory {
            let group = if let Ok(group) =
                read_group_file(virtual_file_system, &mut buffer, group_entry.get_name()).await
            {
                group
            } else {
                // ? : Log error ?
                continue;
            };

            users_manager
                .add_group(group.get_identifier(), group.get_name(), group.get_users())
                .await
                .map_err(Error::FailedToAddGroup)?;
        }
    }

    {
        let users_directory = Directory::open(virtual_file_system, USERS_FOLDER_PATH)
            .await
            .map_err(Error::FailedToReadUsersDirectory)?;

        // Read all users.
        for user_entry in users_directory {
            let user = if let Ok(user) =
                read_user_file(virtual_file_system, &mut buffer, user_entry.get_name()).await
            {
                user
            } else {
                // ? : Log error ?
                continue;
            };

            users_manager
                .add_user(
                    user.get_identifier(),
                    user.get_name(),
                    user.get_primary_group(),
                )
                .await
                .map_err(Error::FailedToAddUser)?;
        }
    }

    Ok(())
}
