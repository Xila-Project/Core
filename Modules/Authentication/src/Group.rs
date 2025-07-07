//! Group management functionality for the Authentication module.
//!
//! This module provides group management capabilities including:
//! - Group creation and management
//! - User group membership tracking
//! - File-based persistent storage of group data
//!
//! All group data is stored as JSON files in the `/System/Groups/` directory,
//! with each group having their own file named after their group name.

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use miniserde::{Deserialize, Serialize};
use File_system::{Flags_type, Mode_type, Open_type, Path_owned_type, Path_type};
use Users::{
    Group_identifier_inner_type, Group_identifier_type, User_identifier_inner_type,
    User_identifier_type,
};
use Virtual_file_system::{Directory_type, File_type, Virtual_file_system_type};

use crate::{Error_type, Result_type, GROUP_FOLDER_PATH};

/// Represents a user group with associated metadata and member list.
///
/// This structure contains all the information needed to represent a group
/// in the system, including its unique identifier, name, and list of users
/// that belong to the group.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group_type {
    /// Unique identifier for the group
    Identifier: Group_identifier_inner_type,
    /// Human-readable group name
    Name: String,
    /// List of user identifiers that belong to this group
    Users: Vec<User_identifier_inner_type>,
}

impl Group_type {
    /// Creates a new group instance with the provided information.
    ///
    /// # Arguments
    ///
    /// * `Identifier` - Unique numerical identifier for the group
    /// * `Name` - Human-readable group name
    /// * `Users` - List of user identifiers that belong to this group
    ///
    /// # Returns
    ///
    /// A new `Group_type` instance with the provided data.
    pub fn New(
        Identifier: Group_identifier_inner_type,
        Name: String,
        Users: Vec<User_identifier_inner_type>,
    ) -> Self {
        Self {
            Identifier,
            Name,
            Users,
        }
    }

    /// Returns the group's unique identifier.
    ///
    /// # Returns
    ///
    /// A `Group_identifier_type` containing the group's unique ID.
    pub fn Get_identifier(&self) -> Group_identifier_type {
        Group_identifier_type::New(self.Identifier)
    }

    /// Returns the group's name as a string slice.
    ///
    /// # Returns
    ///
    /// A string slice containing the group name.
    pub fn Get_name(&self) -> &str {
        &self.Name
    }

    /// Returns the list of users that belong to this group.
    ///
    /// This function uses unsafe transmutation to avoid copying the vector,
    /// since `User_identifier_type` is transparent to `User_identifier_inner_type`.
    ///
    /// # Returns
    ///
    /// A slice of `User_identifier_type` containing all group members.
    pub fn Get_users(&self) -> &[User_identifier_type] {
        // Avoid to copy the vector since User_identifier_type is transparent to User_identifier_inner_type.
        unsafe { core::mem::transmute(self.Users.as_slice()) }
    }
}

/// Constructs the file system path for a group's data file.
///
/// # Arguments
///
/// * `Group_name` - The group name to generate a path for
///
/// # Returns
///
/// Returns `Ok(Path_owned_type)` with the complete path to the group file,
/// or `Err(Error_type::Failed_to_get_group_file_path)` if path construction fails.
pub fn Get_group_file_path(Group_name: &str) -> Result_type<Path_owned_type> {
    Path_type::New(GROUP_FOLDER_PATH)
        .to_owned()
        .Append(Group_name)
        .ok_or(Error_type::Failed_to_get_group_file_path)
}

/// Reads and parses a group file from the filesystem.
///
/// This function is used internally to load group data from JSON files.
/// It reads the file contents into the provided buffer and deserializes
/// the JSON data into a `Group_type` structure.
///
/// # Arguments
///
/// * `Virtual_file_system` - Reference to the virtual file system
/// * `Buffer` - Mutable buffer to use for reading file contents
/// * `File` - Name of the group file to read
///
/// # Returns
///
/// Returns `Ok(Group_type)` with the parsed group data,
/// or an appropriate error if reading or parsing fails.
///
/// # Errors
///
/// - Path construction failures
/// - File system errors (opening, reading)
/// - JSON parsing errors
pub async fn Read_group_file<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Buffer: &mut Vec<u8>,
    File: &str,
) -> Result_type<Group_type> {
    let Group_file_path = Path_type::New(GROUP_FOLDER_PATH)
        .to_owned()
        .Append(File)
        .ok_or(Error_type::Failed_to_get_group_file_path)?;

    let Group_file = File_type::Open(
        Virtual_file_system,
        Group_file_path,
        Mode_type::READ_ONLY.into(),
    )
    .await
    .map_err(Error_type::Failed_to_read_group_directory)?;

    Buffer.clear();

    Group_file
        .Read_to_end(Buffer)
        .await
        .map_err(Error_type::Failed_to_read_group_file)?;

    miniserde::json::from_str(core::str::from_utf8(Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_group_file)
}

/// Creates a new group with the specified parameters.
///
/// This function creates a new group in the system by:
/// 1. Generating a new group identifier (if not provided)
/// 2. Adding the group to the Users manager
/// 3. Creating the group file with the group data
///
/// The group is created with an empty user list initially.
///
/// # Arguments
///
/// * `Virtual_file_system` - Reference to the virtual file system
/// * `Group_name` - Name for the new group
/// * `Group_identifier` - Optional specific group identifier (auto-generated if None)
///
/// # Returns
///
/// Returns `Ok(Group_identifier_type)` with the new group's identifier,
/// or an appropriate error if creation fails.
///
/// # Errors
///
/// This function can fail for various reasons including:
/// - Group identifier generation or assignment failures
/// - File system operations (directory creation, file writing)
/// - Users manager operations (adding group)
pub async fn Create_group<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Group_name: &str,
    Group_identifier: Option<Group_identifier_type>,
) -> Result_type<Group_identifier_type> {
    let Users_manager = Users::Get_instance();

    // - New group identifier if not provided.
    let Group_identifier = if let Some(Group_identifier) = Group_identifier {
        Group_identifier
    } else {
        Users_manager
            .Get_new_group_identifier()
            .await
            .map_err(Error_type::Failed_to_get_new_group_identifier)?
    };

    // - Add it to the users manager.
    Users_manager
        .Add_group(Group_identifier, Group_name, &[])
        .await
        .map_err(Error_type::Failed_to_add_group)?;

    // - Write group file.
    let Group = Group_type::New(Group_identifier.As_u16(), Group_name.to_string(), vec![]);

    match Directory_type::Create(Virtual_file_system, GROUP_FOLDER_PATH).await {
        Ok(_) | Err(File_system::Error_type::Already_exists) => {}
        Err(Error) => Err(Error_type::Failed_to_create_groups_directory(Error))?,
    };

    let Group_file_path = Get_group_file_path(Group_name)?;

    let Group_file = File_type::Open(
        Virtual_file_system,
        Group_file_path,
        Flags_type::New(Mode_type::WRITE_ONLY, Some(Open_type::CREATE_ONLY), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_group_file)?;

    let Group_json = miniserde::json::to_string(&Group);

    Group_file
        .Write(Group_json.as_bytes())
        .await
        .map_err(Error_type::Failed_to_write_group_file)?;

    Ok(Group_identifier)
}
