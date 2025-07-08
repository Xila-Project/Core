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
    Hash::{generate_salt, hash_password},
    Result_type, USERS_FOLDER_PATH,
};

/// Represents a user account with all associated metadata.
///
/// This structure contains all the information needed to represent a user
/// in the system, including their unique identifier, name, primary group,
/// and hashed password with salt for security.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User_type {
    /// Unique identifier for the user
    identifier: User_identifier_inner_type,
    /// Human-readable username
    name: String,
    /// Identifier of the user's primary group
    primary_group: Group_identifier_inner_type,
    /// SHA-512 hash of the user's password combined with salt
    hash: String,
    /// Random salt used for password hashing
    salt: String,
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
    pub fn new(
        identifier: User_identifier_inner_type,
        name: String,
        primary_group: Group_identifier_inner_type,
        hash: String,
        salt: String,
    ) -> Self {
        Self {
            identifier,
            name,
            primary_group,
            hash,
            salt,
        }
    }

    /// Returns the user's unique identifier.
    ///
    /// # Returns
    ///
    /// A `User_identifier_type` containing the user's unique ID.
    pub fn get_identifier(&self) -> User_identifier_type {
        User_identifier_type::New(self.identifier)
    }

    /// Returns the user's primary group identifier.
    ///
    /// # Returns
    ///
    /// A `Group_identifier_type` containing the user's primary group ID.
    pub fn get_primary_group(&self) -> Group_identifier_type {
        Group_identifier_type::New(self.primary_group)
    }

    /// Returns the user's name as a string slice.
    ///
    /// # Returns
    ///
    /// A string slice containing the username.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the user's password hash as a string slice.
    ///
    /// # Returns
    ///
    /// A string slice containing the SHA-512 hash of password+salt.
    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    /// Returns the user's salt as a string slice.
    ///
    /// # Returns
    ///
    /// A string slice containing the random salt used for password hashing.
    pub fn get_salt(&self) -> &str {
        &self.salt
    }

    /// Updates the user's password hash.
    ///
    /// # Arguments
    ///
    /// * `Hash` - New SHA-512 hash to store
    pub fn set_hash(&mut self, Hash: String) {
        self.hash = Hash;
    }

    /// Updates the user's salt.
    ///
    /// # Arguments
    ///
    /// * `Salt` - New salt to store
    pub fn set_salt(&mut self, Salt: String) {
        self.salt = Salt;
    }

    /// Updates the user's primary group.
    ///
    /// # Arguments
    ///
    /// * `Primary_group` - New primary group identifier
    pub fn set_primary_group(&mut self, Primary_group: Group_identifier_inner_type) {
        self.primary_group = Primary_group;
    }

    /// Updates the user's name.
    ///
    /// # Arguments
    ///
    /// * `Name` - New username to store
    pub fn set_name(&mut self, Name: String) {
        self.name = Name;
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
pub fn get_user_file_path(User_name: &str) -> Result_type<Path_owned_type> {
    Path_type::New(USERS_FOLDER_PATH)
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
pub async fn authenticate_user<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
    user_name: &str,
    password: &str,
) -> Result_type<User_identifier_type> {
    let path = get_user_file_path(user_name)?;

    let User_file = File_type::open(virtual_file_system, path, Mode_type::READ_ONLY.into())
        .await
        .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    if hash_password(password, User.get_salt()) == User.get_hash() {
        Ok(User.get_identifier())
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
pub async fn create_user<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
    user_name: &str,
    password: &str,
    primary_group: Group_identifier_type,
    user_identifier: Option<User_identifier_type>,
) -> Result_type<User_identifier_type> {
    let users_manager = Users::get_instance();

    // - New user identifier if not provided.
    let User_identifier = if let Some(User_identifier) = user_identifier {
        User_identifier
    } else {
        users_manager
            .get_new_user_identifier()
            .await
            .map_err(Error_type::Failed_to_get_new_user_identifier)?
    };

    // - Add it to the users manager.
    users_manager
        .Add_user(User_identifier, user_name, primary_group)
        .await
        .map_err(Error_type::Failed_to_create_user)?;

    // - Hash password.
    let Salt = generate_salt().await?;

    let Hash = hash_password(password, &Salt);

    // - Write user file.
    let User = User_type::new(
        User_identifier.As_u16(),
        user_name.to_string(),
        primary_group.As_u16(),
        Hash,
        Salt,
    );

    match Directory_type::create(virtual_file_system, USERS_FOLDER_PATH).await {
        Ok(_) | Err(File_system::Error_type::Already_exists) => {}
        Err(error) => Err(Error_type::Failed_to_create_users_directory(error))?,
    }

    let User_file_path = Path_type::New(USERS_FOLDER_PATH)
        .to_owned()
        .Append(user_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)?;

    let User_file = File_type::open(
        virtual_file_system,
        User_file_path,
        Flags_type::New(Mode_type::WRITE_ONLY, Some(Open_type::CREATE_ONLY), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let User_json = miniserde::json::to_string(&User);

    User_file
        .write(User_json.as_bytes())
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
pub async fn change_user_password<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
    user_name: &str,
    new_password: &str,
) -> Result_type<()> {
    let salt = generate_salt().await?;

    let Hash = hash_password(new_password, &salt);

    let User_file_path = Path_type::New(USERS_FOLDER_PATH)
        .to_owned()
        .Append(user_name)
        .ok_or(Error_type::Failed_to_get_user_file_path)?;

    let User_file = File_type::open(
        virtual_file_system,
        User_file_path,
        Flags_type::New(Mode_type::READ_WRITE, Some(Open_type::TRUNCATE), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let mut User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    User.set_hash(Hash);
    User.set_salt(salt);

    let User_json = miniserde::json::to_string(&User);

    User_file
        .write(User_json.as_bytes())
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
pub async fn change_user_name<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
    current_name: &str,
    new_name: &str,
) -> Result_type<()> {
    let file_path = get_user_file_path(current_name)?;

    let User_file = File_type::open(
        virtual_file_system,
        file_path,
        Flags_type::New(Mode_type::READ_WRITE, Some(Open_type::TRUNCATE), None),
    )
    .await
    .map_err(Error_type::Failed_to_open_user_file)?;

    let mut Buffer = Vec::new();

    User_file
        .read_to_end(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    let mut User: User_type = miniserde::json::from_str(core::str::from_utf8(&Buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)?;

    User.set_name(new_name.to_string());

    let User_json = miniserde::json::to_string(&User);

    User_file
        .write(User_json.as_bytes())
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
pub async fn read_user_file<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
    buffer: &mut Vec<u8>,
    file: &str,
) -> Result_type<User_type> {
    let user_file_path = get_user_file_path(file)?;

    let User_file = File_type::open(
        virtual_file_system,
        user_file_path,
        Mode_type::READ_ONLY.into(),
    )
    .await
    .map_err(Error_type::Failed_to_read_users_directory)?;

    buffer.clear();

    User_file
        .read_to_end(buffer)
        .await
        .map_err(Error_type::Failed_to_read_user_file)?;

    miniserde::json::from_str(core::str::from_utf8(buffer).unwrap())
        .map_err(Error_type::Failed_to_parse_user_file)
}
