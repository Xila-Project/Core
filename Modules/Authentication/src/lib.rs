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
//! - Random salt generation uses `/Devices/Random` device

#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

mod Error;
mod Group;
mod Hash;
mod User;

use alloc::{vec, vec::Vec};
pub use Error::*;
pub use Group::*;
pub use User::*;

/// Path to the users directory in the filesystem
const Users_folder_path: &str = "/System/Users";

/// Path to the groups directory in the filesystem
const Group_folder_path: &str = "/System/Groups";

/// Path to the random device used for salt generation
const Random_device_path: &str = "/Devices/Random";

/// Loads all users and groups from the filesystem into memory.
///
/// This function scans the `/System/Users` and `/System/Groups` directories,
/// reads all user and group files, and adds them to the Users manager.
/// It should be called during system initialization.
///
/// # Returns
///
/// Returns `Ok(())` if all users and groups were loaded successfully,
/// or an `Error_type` if any operation failed.
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
/// use Authentication::Load_all_users_and_groups;
///
/// # async fn example() -> Result<(), Authentication::Error_type> {
/// Load_all_users_and_groups().await?;
/// # Ok(())
/// # }
/// ```
pub async fn Load_all_users_and_groups() -> Result_type<()> {
    use Group::Read_group_file;
    use User::Read_user_file;
    use Virtual_file_system::Directory_type;
    // Open Xila users folder.
    let Virtual_file_system = Virtual_file_system::Get_instance();

    let Users_manager = Users::Get_instance();

    let mut Buffer: Vec<u8> = vec![];

    {
        let Groups_directory = Directory_type::Open(Virtual_file_system, Group_folder_path)
            .await
            .map_err(Error_type::Failed_to_read_group_directory)?;

        // Read all groups.
        for Group_entry in Groups_directory {
            let Group = if let Ok(Group) =
                Read_group_file(Virtual_file_system, &mut Buffer, Group_entry.Get_name()).await
            {
                Group
            } else {
                // ? : Log error ?
                continue;
            };

            Users_manager
                .Add_group(Group.Get_identifier(), Group.Get_name(), Group.Get_users())
                .await
                .map_err(Error_type::Failed_to_add_group)?;
        }
    }

    {
        let Users_directory = Directory_type::Open(Virtual_file_system, Users_folder_path)
            .await
            .map_err(Error_type::Failed_to_read_users_directory)?;

        // Read all users.
        for User_entry in Users_directory {
            let User = if let Ok(User) =
                Read_user_file(Virtual_file_system, &mut Buffer, User_entry.Get_name()).await
            {
                User
            } else {
                // ? : Log error ?
                continue;
            };

            Users_manager
                .Add_user(
                    User.Get_identifier(),
                    User.Get_name(),
                    User.Get_primary_group(),
                )
                .await
                .map_err(Error_type::Failed_to_add_user)?;
        }
    }

    Ok(())
}
