//! Error types and result handling for file system operations.
//!
//! This module defines all possible errors that can occur during file system operations,
//! along with conversion traits and display implementations for comprehensive error reporting.

use core::{fmt::Display, num::NonZeroU32};

/// Standard result type for file system operations.
///
/// This is a convenience alias for `Result<T, Error>` used throughout the file system crate.
/// All file system operations that can fail return this type.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// use file_system::{Result, Error};
///
/// fn example_operation() -> Result<String> {
///     Ok("Success".into())
/// }
///
/// fn failing_operation() -> Result<()> {
///     Err(Error::Permission_denied)
/// }
/// ```
pub type Result<T> = core::result::Result<T, Error>;

/// Comprehensive enumeration of all possible file system errors.
///
/// This enum covers errors that can occur at various levels of the file system stack,
/// from low-level device operations to high-level file system operations. Each variant
/// has a unique numeric discriminant for FFI compatibility.
///
/// # Error Categories
///
/// ## Initialization Errors
/// - [`Failed_to_initialize_file_system`] - File system initialization failed
/// - [`Already_initialized`] - Attempted to initialize already initialized system
/// - [`Not_initialized`] - Operation attempted on uninitialized system
///
/// ## Permission and Access Errors
/// - [`Permission_denied`] - Insufficient permissions for operation
/// - [`Invalid_mode`] - Invalid access mode specified
/// - [`Invalid_flags`] - Invalid flags provided
///
/// ## File and Directory Errors
/// - [`Not_found`] - File or directory doesn't exist
/// - [`Already_exists`] - File or directory already exists
/// - [`Directory_already_exists`] - Directory already exists
/// - [`Invalid_file`] - File is corrupted or invalid
/// - [`Invalid_directory`] - Directory is corrupted or invalid
/// - [`Invalid_symbolic_link`] - Symbolic link is invalid
/// - [`Not_directory`] - Expected directory but found file
/// - [`is_directory`] - Expected file but found directory
/// - [`Directory_not_empty`] - Cannot remove non-empty directory
///
/// ## Resource and Capacity Errors
/// - [`File_system_full`] - No space left on file system
/// - [`No_space_left`] - No space left on device
/// - [`Too_many_mounted_file_systems`] - Mount limit exceeded
/// - [`Too_many_open_files`] - Open file limit exceeded
/// - [`File_too_large`] - File exceeds maximum size
/// - [`No_memory`] - Insufficient memory for operation
///
/// ## I/O and Device Errors
/// - [`Input_output`] - I/O error during operation
/// - [`Corrupted`] - Data corruption detected
/// - [`Ressource_busy`] - Resource temporarily unavailable
/// - [`UnsupportedOperation`] - Operation not supported by device/filesystem
///
/// ## System Integration Errors
/// - [`Failed_to_get_task_informations`] - Task manager access failed
/// - [`Failed_to_get_users_informations`] - User manager access failed
/// - [`Failed_to_get_users_manager_instance`] - User manager instance unavailable
/// - [`Failed_to_get_task_manager_instance`] - Task manager instance unavailable
///
/// ## Parameter and Validation Errors
/// - [`InvalidParameter`] - Invalid parameter provided
/// - [`Invalid_path`] - Path is malformed or invalid
/// - [`Invalid_identifier`] - File or task identifier is invalid
/// - [`Name_too_long`] - File or directory name exceeds limits
/// - [`Invalid_inode`] - Inode reference is invalid
///
/// ## Metadata and Attribute Errors
/// - [`No_attribute`] - Requested attribute doesn't exist
/// - [`Time_error`] - Timestamp operation failed
///
/// ## Generic Errors
/// - [`File_system_error`] - Generic file system error
/// - [`Internal_error`] - Internal implementation error
/// - [`Unknown`] - Unknown or unspecified error
/// - [`Other`] - Other unclassified error
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
#[repr(C)]
pub enum Error {
    /// Failed to initialize the file system.
    FailedToInitializeFileSystem = 1,
    /// Permission denied for the requested operation.
    PermissionDenied,
    /// File, directory, or resource not found.
    NotFound,
    /// File or directory already exists.
    AlreadyExists,
    /// Directory already exists (more specific than Already_exists).
    DirectoryAlreadyExists,
    /// File system is full and cannot store more data.
    FileSystemFull,
    /// Generic file system error.
    FileSystemError,
    /// The provided path is invalid or malformed.
    InvalidPath,
    /// The file is corrupted or invalid.
    InvalidFile,
    /// The directory is corrupted or invalid.
    InvalidDirectory,
    /// The symbolic link is invalid or broken.
    InvalidSymbolicLink,
    /// Unknown or unspecified error.
    Unknown,
    /// File or task identifier is invalid.
    InvalidIdentifier,
    /// Failed to retrieve task information from task manager.
    FailedToGetTaskInformations,
    /// Failed to retrieve user information from user manager.
    FailedToGetUsersInformations,
    /// Maximum number of mounted file systems exceeded.
    TooManyMountedFileSystems,
    /// Maximum number of open files exceeded.
    TooManyOpenFiles,
    /// Internal implementation error.
    InternalError,
    /// Invalid access mode specified.
    InvalidMode,
    /// Operation is not supported by the device or file system.
    UnsupportedOperation,
    /// Resource is temporarily busy or unavailable.
    RessourceBusy,
    /// System or component is already initialized.
    AlreadyInitialized,
    /// System or component is not initialized.
    NotInitialized,
    /// Failed to get users manager instance.
    FailedToGetUsersManagerInstance,
    /// Failed to get task manager instance.
    FailedToGetTaskManagerInstance,
    /// Invalid parameter provided to function.
    InvalidParameter,
    /// Invalid flags specified for operation.
    InvalidFlags,
    /// Expected a directory but found a file.
    NotDirectory,
    /// Expected a file but found a directory.
    IsDirectory,
    /// Input/output error during operation.
    InputOutput,
    /// Directory is not empty and cannot be removed.
    DirectoryNotEmpty,
    /// File size exceeds maximum allowed size.
    FileTooLarge,
    /// Requested attribute does not exist.
    NoAttribute,
    /// File or directory name is too long.
    NameTooLong,
    /// Data corruption detected.
    Corrupted,
    /// Insufficient memory for operation.
    NoMemory,
    /// No space left on device.
    NoSpaceLeft,
    /// Error in timestamp or time-related operation.
    TimeError,
    /// Invalid inode reference.
    InvalidInode,
    /// Other unclassified error.
    Other,
}

impl Error {
    /// Get the numeric discriminant of the error as a non-zero u32.
    ///
    /// This is useful for FFI operations where errors need to be represented
    /// as numeric codes.
    ///
    /// # Returns
    ///
    /// A `NonZeroU32` containing the error's discriminant value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// use file_system::Error;
    ///
    /// let error = Error::Permission_denied;
    /// let code = error.get_discriminant();
    /// assert_eq!(code.get(), 2); // Permission_denied has discriminant 2
    /// ```
    pub fn get_discriminant(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(*self as u32) }
    }
}

/// Convert Task module errors to file system errors.
///
/// This allows transparent handling of task-related errors in file system operations.
impl From<task::Error> for Error {
    fn from(_: task::Error) -> Self {
        Error::FailedToGetTaskInformations
    }
}

/// Convert Users module errors to file system errors.
///
/// This allows transparent handling of user-related errors in file system operations.
impl From<users::Error> for Error {
    fn from(_: users::Error) -> Self {
        Error::FailedToGetUsersInformations
    }
}

/// Convert file system errors to numeric discriminants.
///
/// This conversion is useful for FFI where errors need to be represented as numbers.
impl From<Error> for NonZeroU32 {
    fn from(error: Error) -> Self {
        error.get_discriminant()
    }
}

/// Display implementation for user-friendly error messages.
///
/// Provides human-readable descriptions of all error variants.
impl Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let string = match self {
            Error::FailedToInitializeFileSystem => "Failed to initialize file system",
            Error::PermissionDenied => "Permission denied",
            Error::NotFound => "Not found",
            Error::AlreadyExists => "Already exists",
            Error::DirectoryAlreadyExists => "Directory already exists",
            Error::FileSystemFull => "File system full",
            Error::FileSystemError => "File system error",
            Error::InvalidPath => "Invalid path",
            Error::InvalidFile => "Invalid file",
            Error::InvalidDirectory => "Invalid directory",
            Error::InvalidSymbolicLink => "Invalid symbolic link",
            Error::Unknown => "Unknown",
            Error::InvalidIdentifier => "Invalid identifier",
            Error::FailedToGetTaskInformations => "Failed to get task informations",
            Error::FailedToGetUsersInformations => "Failed to get users informations",
            Error::TooManyMountedFileSystems => "Too many mounted file systems",
            Error::TooManyOpenFiles => "Too many open files",
            Error::InternalError => "Internal error",
            Error::InvalidMode => "Invalid mode",
            Error::UnsupportedOperation => "Unsupported operation",
            Error::RessourceBusy => "Ressource busy",
            Error::AlreadyInitialized => "Already initialized",
            Error::NotInitialized => "Not initialized",
            Error::FailedToGetUsersManagerInstance => "Failed to get users manager instance",
            Error::FailedToGetTaskManagerInstance => "Failed to get task manager instance",
            Error::InvalidParameter => "Invalid parameter",
            Error::InvalidFlags => "Invalid flags",
            Error::NotDirectory => "Not directory",
            Error::IsDirectory => "Is directory",
            Error::InputOutput => "Input output",
            Error::DirectoryNotEmpty => "Directory not empty",
            Error::FileTooLarge => "File too large",
            Error::NoAttribute => "No attribute",
            Error::NameTooLong => "Name too long",
            Error::Corrupted => "Corrupted",
            Error::NoMemory => "No memory",
            Error::NoSpaceLeft => "No space left",
            Error::TimeError => "Time error",
            Error::InvalidInode => "Invalid inode",
            Error::Other => "Other",
        };

        write!(formatter, "{string}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_error_discriminants() {
        // Test that each error has a unique discriminant
        assert_eq!(
            Error::FailedToInitializeFileSystem.get_discriminant().get(),
            1
        );
        assert_eq!(Error::PermissionDenied.get_discriminant().get(), 2);
        assert_eq!(Error::NotFound.get_discriminant().get(), 3);
        assert_eq!(Error::AlreadyExists.get_discriminant().get(), 4);
        assert_eq!(Error::DirectoryAlreadyExists.get_discriminant().get(), 5);

        // Test a few more to ensure discriminants are sequential
        assert_eq!(Error::FileSystemFull.get_discriminant().get(), 6);
        assert_eq!(Error::FileSystemError.get_discriminant().get(), 7);
        assert_eq!(Error::InvalidPath.get_discriminant().get(), 8);
    }

    #[test]
    fn test_error_display() {
        // Test display formatting for all error types
        assert_eq!(
            format!("{}", Error::FailedToInitializeFileSystem),
            "Failed to initialize file system"
        );
        assert_eq!(format!("{}", Error::PermissionDenied), "Permission denied");
        assert_eq!(format!("{}", Error::NotFound), "Not found");
        assert_eq!(format!("{}", Error::AlreadyExists), "Already exists");
        assert_eq!(
            format!("{}", Error::DirectoryAlreadyExists),
            "Directory already exists"
        );
        assert_eq!(format!("{}", Error::FileSystemFull), "File system full");
        assert_eq!(format!("{}", Error::FileSystemError), "File system error");
        assert_eq!(format!("{}", Error::InvalidPath), "Invalid path");
        assert_eq!(format!("{}", Error::InvalidFile), "Invalid file");
        assert_eq!(format!("{}", Error::InvalidDirectory), "Invalid directory");
        assert_eq!(
            format!("{}", Error::InvalidSymbolicLink),
            "Invalid symbolic link"
        );
        assert_eq!(format!("{}", Error::Unknown), "Unknown");
        assert_eq!(
            format!("{}", Error::InvalidIdentifier),
            "Invalid identifier"
        );
        assert_eq!(
            format!("{}", Error::FailedToGetTaskInformations),
            "Failed to get task informations"
        );
        assert_eq!(
            format!("{}", Error::FailedToGetUsersInformations),
            "Failed to get users informations"
        );
        assert_eq!(
            format!("{}", Error::TooManyMountedFileSystems),
            "Too many mounted file systems"
        );
        assert_eq!(
            format!("{}", Error::TooManyOpenFiles),
            "Too many open files"
        );
        assert_eq!(format!("{}", Error::InternalError), "Internal error");
        assert_eq!(format!("{}", Error::InvalidMode), "Invalid mode");
        assert_eq!(
            format!("{}", Error::UnsupportedOperation),
            "Unsupported operation"
        );
        assert_eq!(format!("{}", Error::RessourceBusy), "Ressource busy");
        assert_eq!(
            format!("{}", Error::AlreadyInitialized),
            "Already initialized"
        );
        assert_eq!(format!("{}", Error::NotInitialized), "Not initialized");
        assert_eq!(
            format!("{}", Error::FailedToGetUsersManagerInstance),
            "Failed to get users manager instance"
        );
        assert_eq!(
            format!("{}", Error::FailedToGetTaskManagerInstance),
            "Failed to get task manager instance"
        );
        assert_eq!(format!("{}", Error::InvalidParameter), "Invalid parameter");
        assert_eq!(format!("{}", Error::InvalidFlags), "Invalid flags");
        assert_eq!(format!("{}", Error::NotDirectory), "Not directory");
        assert_eq!(format!("{}", Error::IsDirectory), "Is directory");
        assert_eq!(format!("{}", Error::InputOutput), "Input output");
        assert_eq!(
            format!("{}", Error::DirectoryNotEmpty),
            "Directory not empty"
        );
        assert_eq!(format!("{}", Error::FileTooLarge), "File too large");
        assert_eq!(format!("{}", Error::NoAttribute), "No attribute");
        assert_eq!(format!("{}", Error::NameTooLong), "Name too long");
        assert_eq!(format!("{}", Error::Corrupted), "Corrupted");
        assert_eq!(format!("{}", Error::NoMemory), "No memory");
        assert_eq!(format!("{}", Error::NoSpaceLeft), "No space left");
        assert_eq!(format!("{}", Error::TimeError), "Time error");
        assert_eq!(format!("{}", Error::InvalidInode), "Invalid inode");
        assert_eq!(format!("{}", Error::Other), "Other");
    }

    #[test]
    fn test_error_debug() {
        // Test debug formatting
        let error = Error::PermissionDenied;
        let debug_str = format!("{error:?}");
        assert_eq!(debug_str, "Permission_denied");
    }

    #[test]
    fn test_error_equality() {
        // Test equality and cloning
        let error1 = Error::NotFound;
        let error2 = Error::NotFound;
        let error3 = Error::PermissionDenied;

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);

        let cloned = error1;
        assert_eq!(error1, cloned);
    }

    #[test]
    fn test_error_conversions() {
        // Test conversion to NonZeroU32
        let error = Error::NotFound;
        let discriminant: NonZeroU32 = error.into();
        assert_eq!(discriminant.get(), 3);

        // Test explicit discriminant access
        assert_eq!(error.get_discriminant().get(), 3);
    }

    #[test]
    fn test_result_type() {
        // Test the Result alias
        let success: Result<i32> = Ok(42);
        let failure: Result<i32> = Err(Error::PermissionDenied);

        assert_eq!(success, Ok(42));

        assert_eq!(failure, Err(Error::PermissionDenied));
    }

    #[test]
    fn test_error_categories() {
        // Test that errors can be categorized by their discriminant ranges

        // Initialization errors (1-3 range roughly)
        assert!(matches!(
            Error::FailedToInitializeFileSystem.get_discriminant().get(),
            1
        ));
        assert!(matches!(
            Error::AlreadyInitialized.get_discriminant().get(),
            22
        ));
        assert!(matches!(Error::NotInitialized.get_discriminant().get(), 23));

        // Permission errors
        assert!(matches!(
            Error::PermissionDenied.get_discriminant().get(),
            2
        ));
        assert!(matches!(Error::InvalidMode.get_discriminant().get(), 19));

        // File/Directory errors
        assert!(matches!(Error::NotFound.get_discriminant().get(), 3));
        assert!(matches!(Error::AlreadyExists.get_discriminant().get(), 4));
        assert!(matches!(
            Error::DirectoryAlreadyExists.get_discriminant().get(),
            5
        ));
    }

    #[test]
    fn test_error_copy_semantics() {
        // Test that Error implements Copy
        let error = Error::FileSystemFull;
        let copied = error; // This should work due to Copy trait

        // Both should be usable
        assert_eq!(error, Error::FileSystemFull);
        assert_eq!(copied, Error::FileSystemFull);
        assert_eq!(error, copied);
    }

    #[test]
    fn test_error_size() {
        // Ensure Error has a reasonable size for an enum
        use core::mem::size_of;

        // Should be small since it's a C-style enum
        assert!(size_of::<Error>() <= 4); // Should be 4 bytes or less
    }

    #[test]
    fn test_nonzero_conversion() {
        // Test that all errors convert to valid NonZeroU32
        let errors = [
            Error::FailedToInitializeFileSystem,
            Error::PermissionDenied,
            Error::NotFound,
            Error::AlreadyExists,
            Error::DirectoryAlreadyExists,
            Error::FileSystemFull,
            Error::FileSystemError,
            Error::InvalidPath,
            Error::InvalidFile,
            Error::InvalidDirectory,
            Error::InvalidSymbolicLink,
            Error::Unknown,
            Error::InvalidIdentifier,
            Error::FailedToGetTaskInformations,
            Error::FailedToGetUsersInformations,
            Error::TooManyMountedFileSystems,
            Error::TooManyOpenFiles,
            Error::InternalError,
            Error::InvalidMode,
            Error::UnsupportedOperation,
            Error::RessourceBusy,
            Error::AlreadyInitialized,
            Error::NotInitialized,
            Error::FailedToGetUsersManagerInstance,
            Error::FailedToGetTaskManagerInstance,
            Error::InvalidParameter,
            Error::InvalidFlags,
            Error::NotDirectory,
            Error::IsDirectory,
            Error::InputOutput,
            Error::DirectoryNotEmpty,
            Error::FileTooLarge,
            Error::NoAttribute,
            Error::NameTooLong,
            Error::Corrupted,
            Error::NoMemory,
            Error::NoSpaceLeft,
            Error::TimeError,
            Error::InvalidInode,
            Error::Other,
        ];

        for error in errors.iter() {
            let discriminant = error.get_discriminant();
            assert!(discriminant.get() > 0);

            let converted: NonZeroU32 = (*error).into();
            assert_eq!(discriminant, converted);
        }
    }
}
