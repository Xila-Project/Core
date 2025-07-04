//! Password hashing and salt generation utilities.
//!
//! This module provides secure password hashing functionality using SHA-512
//! with random salt generation. It ensures that passwords are never stored
//! in plain text and that rainbow table attacks are ineffective due to
//! unique salts for each password.
//!
//! ## Security Features
//!
//! - **SHA-512 Hashing**: Uses the SHA-512 cryptographic hash function
//! - **Random Salt Generation**: Each password gets a unique 16-byte salt
//! - **Salt Source**: Uses the system's `/Devices/Random` device for entropy
//! - **No Plain Text Storage**: Passwords are only stored as hash+salt combinations

use alloc::{
    format,
    string::{String, ToString},
};
use File_system::Mode_type;
use Virtual_file_system::File_type;

use crate::{Error_type, Random_device_path, Result_type};

/// Generates a random salt for password hashing.
///
/// This function reads 16 bytes of random data from the system's random device
/// and converts them to a readable string format. The salt is used to ensure
/// that identical passwords result in different hashes.
///
/// # Returns
///
/// Returns `Ok(String)` containing a 16-character random salt,
/// or an appropriate error if random data generation fails.
///
/// # Errors
///
/// - `Failed_to_open_random_device` - Cannot access the random device
/// - `Failed_to_read_random_device` - I/O error reading random data
///
/// # Security Note
///
/// The salt generation converts random bytes to lowercase letters (a-z)
/// for readability while maintaining sufficient entropy for security.
pub async fn Generate_salt() -> Result_type<String> {
    let Random_file = File_type::Open(
        Virtual_file_system::Get_instance(),
        Random_device_path,
        Mode_type::Read_only.into(),
    )
    .await
    .map_err(Error_type::Failed_to_open_random_device)?;

    let mut Buffer = [0_u8; 16];

    Random_file
        .Read(&mut Buffer)
        .await
        .map_err(Error_type::Failed_to_read_random_device)?;

    Buffer.iter_mut().for_each(|Byte| {
        *Byte = *Byte % 26 + 97;
    });

    Ok(core::str::from_utf8(&Buffer).unwrap().to_string())
}

/// Computes the SHA-512 hash of a password combined with a salt.
///
/// This function creates a secure hash by concatenating the password and salt,
/// then computing the SHA-512 hash of the combined data. The result is returned
/// as a hexadecimal string.
///
/// # Arguments
///
/// * `Password` - The plain text password to hash
/// * `Salt` - The random salt to combine with the password
///
/// # Returns
///
/// A hexadecimal string representation of the SHA-512 hash.
///
/// # Security Note
///
/// This function uses SHA-512, which is cryptographically secure and resistant
/// to collision attacks. The salt prevents rainbow table attacks and ensures
/// that identical passwords have different hashes.
pub fn Hash_password(Password: &str, Salt: &str) -> String {
    use sha2::Digest;

    let mut Hasher = sha2::Sha512::new();

    Hasher.update(Password.as_bytes());
    Hasher.update(Salt.as_bytes());

    let Hash = Hasher.finalize();

    format!("{Hash:x}")
}
