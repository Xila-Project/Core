//! User management functionality for the Authentication module.
//!
//! This module provides comprehensive user account management including:
//! - User creation and authentication
//! - Password management with secure hashing
//! - User profile management (name, primary group)
//! - File-based persistent storage of user data
//!
//! All user data is stored as JSON files in the `/System/Users/` directory,
//! with each user having their own file named after their username.

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use miniserde::{Deserialize, Serialize};
use File_system::{Flags_type, Mode_type, Open_type, Path_owned_type, Path_type};
use Users::{
    Group_identifier_inner_type, Group_identifier_type, User_identifier_inner_type,
    User_identifier_type,
};
use Virtual_file_system::{Directory_type, File_type, Virtual_file_system_type};

use crate::{
    Error_type,
    Hash::{Generate_salt, Hash_password},
    Result_type, Users_folder_path,
};

/// Represents a user account with all associated metadata.
///
/// This structure contains all the information needed to represent a user
/// in the system, including their unique identifier, name, primary group,
/// and hashed password with salt for security.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User_type {
    /// Unique identifier for the user
    Identifier: User_identifier_inner_type,
    /// Human-readable username
    Name: String,
    /// Identifier of the user's primary group
    Primary_group: Group_identifier_inner_type,
    /// SHA-512 hash of the user's password combined with salt
    Hash: String,
    /// Random salt used for password hashing
    Salt: String,
}

impl User_type {
    /// Creates a new user instance with the provided information.
    ///
    /// # Arguments
    ///
    /// * `Identifier` - Unique numerical identifier for the user
    /// * `Name` - Human-readable username
    /// * `Primary_group` - Identifier of the user's primary group
    /// * `Hash` - Pre-computed SHA-512 hash of password+salt
    /// * `Salt` - Random salt used for password hashing
    ///
    /// # Returns
    ///
    /// A new `User_type` instance with the provided data.
    pub fn New(
        Identifier: User_identifier_inner_type,
        Name: String,
        Primary_group: Group_identifier_inner_type,
        Hash: String,
        Salt: String,
    ) -> Self {
        Self {
            Identifier,
            Name,
            Primary_group,
            Hash,
            Salt,
        }
    }

    /// Returns the user's unique identifier.
    ///
    /// # Returns
    ///
    /// A `User_identifier_type` containing the user's unique ID.
    pub fn Get_identifier(&self) -> User_identifier_type {
        User_identifier_type::New(self.Identifier)
    }

    /// Returns the user's primary group identifier.
    ///
    /// # Returns
    ///
    /// A `Group_identifier_type` containing the user's primary group ID.
    pub fn Get_primary_group(&self) -> Group_identifier_type {
        Group_identifier_type::New(self.Primary_group)
    }

    /// Returns the user's name as a string slice.
    ///
    /// # Returns
    ///
    /// A string slice containing the username.
    pub fn Get_name(&self) -> &str {
        &self.Name
    }

    /// Returns the user's password hash as a string slice.
    ///
    /// # Returns
    ///
    /// A string slice containing the SHA-512 hash of password+salt.
    pub fn Get_hash(&self) -> &str {
        &self.Hash
    }

    /// Returns the user's salt as a string slice.
    ///
    /// # Returns
    ///
    /// A string slice containing the random salt used for password hashing.
    pub fn Get_salt(&self) -> &str {
        &self.Salt
    }

    /// Updates the user's password hash.
    ///
    /// # Arguments
    ///
    /// * `Hash` - New SHA-512 hash to store
    pub fn Set_hash(&mut self, Hash: String) {
        self.Hash = Hash;
    }

    /// Updates the user's salt.
    ///
    /// # Arguments
    ///
    /// * `Salt` - New salt to store
    pub fn Set_salt(&mut self, Salt: String) {
        self.Salt = Salt;
    }

    /// Updates the user's primary group.
    ///
    /// # Arguments
    ///
    /// * `Primary_group` - New primary group identifier
    pub fn Set_primary_group(&mut self, Primary_group: Group_identifier_inner_type) {
        self.Primary_group = Primary_group;
    }

    /// Updates the user's name.
    ///
    /// # Arguments
    ///
    /// * `Name` - New username to store
    pub fn Set_name(&mut self, Name: String) {
        self.Name = Name;
    }
}

/// Constructs the file system path for a user's data file.
///
/// # Arguments
///
/// * `User_name` - The username to generate a path for
///
/// # Returns
///
/// Returns `Ok(Path_owned_type)` with the complete path to the user file,
/// or `Err(Error_type::Failed_to_get_user_file_path)` if path construction fails.
pub fn Get_user_file_path(User_name: &str) -> Result_type<Path_owned_type> {
    Path_type::New(Users_folder_path)
        .to_owned()
        .Append(User_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)
}

/// Authenticates a user with their username and password.
///
/// This function reads the user's file from the filesystem, compares the
/// provided password hash with the stored hash, and returns the user's
/// identifier if authentication succeeds.
///
/// # Arguments
///
/// * `Virtual_file_system` - Reference to the virtual file system
/// * `User_name` - Username to authenticate
/// * `Password` - Plain text password to verify
///
/// # Returns
///
/// Returns `Ok(User_identifier_type)` if authentication succeeds,
/// or an appropriate error if authentication fails or file operations fail.
///
/// # Errors
///
/// - `Failed_to_get_user_file_path` - Invalid username or path construction failure
/// - `Failed_to_open_user_file` - User file doesn't exist or permission denied
/// - `Failed_to_read_user_file` - I/O error reading user file
/// - `Failed_to_parse_user_file` - Invalid JSON format in user file
/// - `Invalid_password` - Password doesn't match stored hash
pub async fn Authenticate_user<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    Password: &str,
) -> Result_type<User_identifier_type> {
    let Path = Get_user_file_path(User_name)?;

    let User_file = File_type::Open(Virtual_file_system, Path, Mode_type::Read_only.into())
        .await
        .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    if Hash_password(Password, User.Get_salt()) == User.Get_hash() {
        Ok(User.Get_identifier())
    } else {
        Err(Error_type::Invalid_password)
    }
}

/// Creates a new user account with the specified parameters.
///
/// This function creates a new user in the system by:
/// 1. Generating a new user identifier (if not provided)
/// 2. Adding the user to the Users manager
/// 3. Generating a random salt and hashing the password
/// 4. Creating the user file with all user data
///
/// # Arguments
///
/// * `Virtual_file_system` - Reference to the virtual file system
/// * `User_name` - Username for the new account
/// * `Password` - Plain text password for the new account
/// * `Primary_group` - Primary group identifier for the user
/// * `User_identifier` - Optional specific user identifier (auto-generated if None)
///
/// # Returns
///
/// Returns `Ok(User_identifier_type)` with the new user's identifier,
/// or an appropriate error if creation fails.
///
/// # Errors
///
/// This function can fail for various reasons including:
/// - User identifier generation or assignment failures
/// - File system operations (directory creation, file writing)
/// - Users manager operations (adding user)
/// - Random salt generation failures
pub async fn Create_user<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    Password: &str,
    Primary_group: Group_identifier_type,
    User_identifier: Option<User_identifier_type>,
) -> Result_type<User_identifier_type> {
    let Users_manager = Users::Get_instance();

    // - New user identifier if not provided.
    let User_identifier = if let Some(User_identifier) = User_identifier {
        User_identifier
    } else {
        Users_manager
            .Get_new_user_identifier()
            .await
            .map_err(Error_type::Failed_to_get_new_user_identifier)?
    };

    // - Add it to the users manager.
    Users_manager
        .Add_user(User_identifier, User_name, Primary_group)
        .await
        .map_err(Error_type::Failed_to_create_user)?;

    // - Hash password.
    let Salt = Generate_salt().await?;

    let Hash = Hash_password(Password, &Salt);

    // - Write user file.
    let User = User_type::New(
        User_identifier.As_u16(),
        User_name.to_string(),
        Primary_group.As_u16(),
        Hash,
        Salt,
    );

    match Directory_type::Create(Virtual_file_system, Users_folder_path).await {
        Ok(_) | Err(File_system::Error_type::Already_exists) => {}
        Err(Error) => Err(Error_type::Failed_to_create_users_directory(Error))?,
    }

    let User_file_path = Path_type::New(Users_folder_path)
        .to_owned()
        .Append(User_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        User_file_path,
        Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .await
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(User_identifier)
}

/// Changes a user's password by generating a new salt and hash.
///
/// This function updates a user's password by:
/// 1. Generating a new random salt
/// 2. Hashing the new password with the salt
/// 3. Reading the existing user file
/// 4. Updating the hash and salt fields
/// 5. Writing the updated data back to the file
///
/// # Arguments
///
/// * `Virtual_file_system` - Reference to the virtual file system
/// * `User_name` - Username of the account to update
/// * `New_password` - New plain text password
///
/// # Returns
///
/// Returns `Ok(())` if the password was changed successfully,
/// or an appropriate error if the operation fails.
///
/// # Errors
///
/// - File system errors (opening, reading, writing user file)
/// - Salt generation failures
/// - JSON parsing errors
pub async fn Change_user_password<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    User_name: &str,
    New_password: &str,
) -> Result_type<()> {
    let Salt = Generate_salt().await?;

    let Hash = Hash_password(New_password, &Salt);

    let User_file_path = Path_type::New(Users_folder_path)
        .to_owned()
        .Append(User_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        User_file_path,
        Flags_type::New(Mode_type::Read_write, Some(Open_type::Truncate), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let mut User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    User.Set_hash(Hash);
    User.Set_salt(Salt);

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .await
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(())
}

/// Changes a user's username by updating their user file.
///
/// This function reads the user's existing data, updates the name field,
/// and writes the modified data back to the file system.
///
/// # Arguments
///
/// * `Virtual_file_system` - Reference to the virtual file system
/// * `Current_name` - Current username of the account
/// * `New_name` - New username to assign
///
/// # Returns
///
/// Returns `Ok(())` if the username was changed successfully,
/// or an appropriate error if the operation fails.
///
/// # Errors
///
/// - File system errors (opening, reading, writing user file)
/// - JSON parsing errors
/// - Path construction failures
pub async fn Change_user_name<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Current_name: &str,
    New_name: &str,
) -> Result_type<()> {
    let File_path = Get_user_file_path(Current_name)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        File_path,
        Flags_type::New(Mode_type::Read_write, Some(Open_type::Truncate), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .Read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let mut User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    User.Set_name(New_name.to_string());

    let User_json = miniserde::json::to_string(&User);

    User_file
        .Write(User_json.as_bytes())
        .await
        .map_err(Error_type::Failed_to_write_user_file)?;

    Ok(())
}

/// Reads and parses a user file from the filesystem.
///
/// This function is used internally to load user data from JSON files.
/// It reads the file contents into the provided buffer and deserializes
/// the JSON data into a `User_type` structure.
///
/// # Arguments
///
/// * `Virtual_file_system` - Reference to the virtual file system
/// * `Buffer` - Mutable buffer to use for reading file contents
/// * `File` - Name of the user file to read
///
/// # Returns
///
/// Returns `Ok(User_type)` with the parsed user data,
/// or an appropriate error if reading or parsing fails.
///
/// # Errors
///
/// - Path construction failures
/// - File system errors (opening, reading)
/// - JSON parsing errors
pub async fn Read_user_file<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Buffer: &mut Vec<u8>,
    File: &str,
) -> Result_type<User_type> {
    let User_file_path = Get_user_file_path(File)?;

    let User_file = File_type::Open(
        Virtual_file_system,
        User_file_path,
        Mode_type::Read_only.into(),
    )
    .await
    .map_err(Error_type::Failed_to_read_users_directory)?;

    Buffer.clear();

    User_file
        .Read_to_end(Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    miniserde::json::from_str(core::str::from_utf8(Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)
}
