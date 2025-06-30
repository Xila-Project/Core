//! Error types and result handling for file system operations.
//!
//! This module defines all possible errors that can occur during file system operations,
//! along with conversion traits and display implementations for comprehensive error reporting.

use core::{fmt::Display, num::NonZeroU32};

/// Standard result type for file system operations.
///
/// This is a convenience alias for `Result<T, Error_type>` used throughout the file system crate.
/// All file system operations that can fail return this type.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// use File_system::{Result_type, Error_type};
///
/// fn example_operation() -> Result_type<String> {
///     Ok("Success".into())
/// }
///
/// fn failing_operation() -> Result_type<()> {
///     Err(Error_type::Permission_denied)
/// }
/// ```
pub type Result_type<T> = core::result::Result<T, Error_type>;

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
/// - [`Is_directory`] - Expected file but found directory
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
/// - [`Unsupported_operation`] - Operation not supported by device/filesystem
///
/// ## System Integration Errors
/// - [`Failed_to_get_task_informations`] - Task manager access failed
/// - [`Failed_to_get_users_informations`] - User manager access failed
/// - [`Failed_to_get_users_manager_instance`] - User manager instance unavailable
/// - [`Failed_to_get_task_manager_instance`] - Task manager instance unavailable
///
/// ## Parameter and Validation Errors
/// - [`Invalid_parameter`] - Invalid parameter provided
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
pub enum Error_type {
    /// Failed to initialize the file system.
    Failed_to_initialize_file_system = 1,
    /// Permission denied for the requested operation.
    Permission_denied,
    /// File, directory, or resource not found.
    Not_found,
    /// File or directory already exists.
    Already_exists,
    /// Directory already exists (more specific than Already_exists).
    Directory_already_exists,
    /// File system is full and cannot store more data.
    File_system_full,
    /// Generic file system error.
    File_system_error,
    /// The provided path is invalid or malformed.
    Invalid_path,
    /// The file is corrupted or invalid.
    Invalid_file,
    /// The directory is corrupted or invalid.
    Invalid_directory,
    /// The symbolic link is invalid or broken.
    Invalid_symbolic_link,
    /// Unknown or unspecified error.
    Unknown,
    /// File or task identifier is invalid.
    Invalid_identifier,
    /// Failed to retrieve task information from task manager.
    Failed_to_get_task_informations,
    /// Failed to retrieve user information from user manager.
    Failed_to_get_users_informations,
    /// Maximum number of mounted file systems exceeded.
    Too_many_mounted_file_systems,
    /// Maximum number of open files exceeded.
    Too_many_open_files,
    /// Internal implementation error.
    Internal_error,
    /// Invalid access mode specified.
    Invalid_mode,
    /// Operation is not supported by the device or file system.
    Unsupported_operation,
    /// Resource is temporarily busy or unavailable.
    Ressource_busy,
    /// System or component is already initialized.
    Already_initialized,
    /// System or component is not initialized.
    Not_initialized,
    /// Failed to get users manager instance.
    Failed_to_get_users_manager_instance,
    /// Failed to get task manager instance.
    Failed_to_get_task_manager_instance,
    /// Invalid parameter provided to function.
    Invalid_parameter,
    /// Invalid flags specified for operation.
    Invalid_flags,
    /// Expected a directory but found a file.
    Not_directory,
    /// Expected a file but found a directory.
    Is_directory,
    /// Input/output error during operation.
    Input_output,
    /// Directory is not empty and cannot be removed.
    Directory_not_empty,
    /// File size exceeds maximum allowed size.
    File_too_large,
    /// Requested attribute does not exist.
    No_attribute,
    /// File or directory name is too long.
    Name_too_long,
    /// Data corruption detected.
    Corrupted,
    /// Insufficient memory for operation.
    No_memory,
    /// No space left on device.
    No_space_left,
    /// Error in timestamp or time-related operation.
    Time_error,
    /// Invalid inode reference.
    Invalid_inode,
    /// Other unclassified error.
    Other,
}

impl Error_type {
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
    /// use File_system::Error_type;
    ///
    /// let error = Error_type::Permission_denied;
    /// let code = error.Get_discriminant();
    /// assert_eq!(code.get(), 2); // Permission_denied has discriminant 2
    /// ```
    pub fn Get_discriminant(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(*self as u32) }
    }
}

/// Convert Task module errors to file system errors.
///
/// This allows transparent handling of task-related errors in file system operations.
impl From<Task::Error_type> for Error_type {
    fn from(_: Task::Error_type) -> Self {
        Error_type::Failed_to_get_task_informations
    }
}

/// Convert Users module errors to file system errors.
///
/// This allows transparent handling of user-related errors in file system operations.
impl From<Users::Error_type> for Error_type {
    fn from(_: Users::Error_type) -> Self {
        Error_type::Failed_to_get_users_informations
    }
}

/// Convert file system errors to numeric discriminants.
///
/// This conversion is useful for FFI where errors need to be represented as numbers.
impl From<Error_type> for NonZeroU32 {
    fn from(Error: Error_type) -> Self {
        Error.Get_discriminant()
    }
}

/// Display implementation for user-friendly error messages.
///
/// Provides human-readable descriptions of all error variants.
impl Display for Error_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let String = match self {
            Error_type::Failed_to_initialize_file_system => "Failed to initialize file system",
            Error_type::Permission_denied => "Permission denied",
            Error_type::Not_found => "Not found",
            Error_type::Already_exists => "Already exists",
            Error_type::Directory_already_exists => "Directory already exists",
            Error_type::File_system_full => "File system full",
            Error_type::File_system_error => "File system error",
            Error_type::Invalid_path => "Invalid path",
            Error_type::Invalid_file => "Invalid file",
            Error_type::Invalid_directory => "Invalid directory",
            Error_type::Invalid_symbolic_link => "Invalid symbolic link",
            Error_type::Unknown => "Unknown",
            Error_type::Invalid_identifier => "Invalid identifier",
            Error_type::Failed_to_get_task_informations => "Failed to get task informations",
            Error_type::Failed_to_get_users_informations => "Failed to get users informations",
            Error_type::Too_many_mounted_file_systems => "Too many mounted file systems",
            Error_type::Too_many_open_files => "Too many open files",
            Error_type::Internal_error => "Internal error",
            Error_type::Invalid_mode => "Invalid mode",
            Error_type::Unsupported_operation => "Unsupported operation",
            Error_type::Ressource_busy => "Ressource busy",
            Error_type::Already_initialized => "Already initialized",
            Error_type::Not_initialized => "Not initialized",
            Error_type::Failed_to_get_users_manager_instance => {
                "Failed to get users manager instance"
            }
            Error_type::Failed_to_get_task_manager_instance => {
                "Failed to get task manager instance"
            }
            Error_type::Invalid_parameter => "Invalid parameter",
            Error_type::Invalid_flags => "Invalid flags",
            Error_type::Not_directory => "Not directory",
            Error_type::Is_directory => "Is directory",
            Error_type::Input_output => "Input output",
            Error_type::Directory_not_empty => "Directory not empty",
            Error_type::File_too_large => "File too large",
            Error_type::No_attribute => "No attribute",
            Error_type::Name_too_long => "Name too long",
            Error_type::Corrupted => "Corrupted",
            Error_type::No_memory => "No memory",
            Error_type::No_space_left => "No space left",
            Error_type::Time_error => "Time error",
            Error_type::Invalid_inode => "Invalid inode",
            Error_type::Other => "Other",
        };

        write!(Formatter, "{String}")
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use alloc::format;

    #[test]
    fn Test_error_discriminants() {
        // Test that each error has a unique discriminant
        assert_eq!(
            Error_type::Failed_to_initialize_file_system
                .Get_discriminant()
                .get(),
            1
        );
        assert_eq!(Error_type::Permission_denied.Get_discriminant().get(), 2);
        assert_eq!(Error_type::Not_found.Get_discriminant().get(), 3);
        assert_eq!(Error_type::Already_exists.Get_discriminant().get(), 4);
        assert_eq!(
            Error_type::Directory_already_exists
                .Get_discriminant()
                .get(),
            5
        );

        // Test a few more to ensure discriminants are sequential
        assert_eq!(Error_type::File_system_full.Get_discriminant().get(), 6);
        assert_eq!(Error_type::File_system_error.Get_discriminant().get(), 7);
        assert_eq!(Error_type::Invalid_path.Get_discriminant().get(), 8);
    }

    #[test]
    fn Test_error_display() {
        // Test display formatting for all error types
        assert_eq!(
            format!("{}", Error_type::Failed_to_initialize_file_system),
            "Failed to initialize file system"
        );
        assert_eq!(
            format!("{}", Error_type::Permission_denied),
            "Permission denied"
        );
        assert_eq!(format!("{}", Error_type::Not_found), "Not found");
        assert_eq!(format!("{}", Error_type::Already_exists), "Already exists");
        assert_eq!(
            format!("{}", Error_type::Directory_already_exists),
            "Directory already exists"
        );
        assert_eq!(
            format!("{}", Error_type::File_system_full),
            "File system full"
        );
        assert_eq!(
            format!("{}", Error_type::File_system_error),
            "File system error"
        );
        assert_eq!(format!("{}", Error_type::Invalid_path), "Invalid path");
        assert_eq!(format!("{}", Error_type::Invalid_file), "Invalid file");
        assert_eq!(
            format!("{}", Error_type::Invalid_directory),
            "Invalid directory"
        );
        assert_eq!(
            format!("{}", Error_type::Invalid_symbolic_link),
            "Invalid symbolic link"
        );
        assert_eq!(format!("{}", Error_type::Unknown), "Unknown");
        assert_eq!(
            format!("{}", Error_type::Invalid_identifier),
            "Invalid identifier"
        );
        assert_eq!(
            format!("{}", Error_type::Failed_to_get_task_informations),
            "Failed to get task informations"
        );
        assert_eq!(
            format!("{}", Error_type::Failed_to_get_users_informations),
            "Failed to get users informations"
        );
        assert_eq!(
            format!("{}", Error_type::Too_many_mounted_file_systems),
            "Too many mounted file systems"
        );
        assert_eq!(
            format!("{}", Error_type::Too_many_open_files),
            "Too many open files"
        );
        assert_eq!(format!("{}", Error_type::Internal_error), "Internal error");
        assert_eq!(format!("{}", Error_type::Invalid_mode), "Invalid mode");
        assert_eq!(
            format!("{}", Error_type::Unsupported_operation),
            "Unsupported operation"
        );
        assert_eq!(format!("{}", Error_type::Ressource_busy), "Ressource busy");
        assert_eq!(
            format!("{}", Error_type::Already_initialized),
            "Already initialized"
        );
        assert_eq!(
            format!("{}", Error_type::Not_initialized),
            "Not initialized"
        );
        assert_eq!(
            format!("{}", Error_type::Failed_to_get_users_manager_instance),
            "Failed to get users manager instance"
        );
        assert_eq!(
            format!("{}", Error_type::Failed_to_get_task_manager_instance),
            "Failed to get task manager instance"
        );
        assert_eq!(
            format!("{}", Error_type::Invalid_parameter),
            "Invalid parameter"
        );
        assert_eq!(format!("{}", Error_type::Invalid_flags), "Invalid flags");
        assert_eq!(format!("{}", Error_type::Not_directory), "Not directory");
        assert_eq!(format!("{}", Error_type::Is_directory), "Is directory");
        assert_eq!(format!("{}", Error_type::Input_output), "Input output");
        assert_eq!(
            format!("{}", Error_type::Directory_not_empty),
            "Directory not empty"
        );
        assert_eq!(format!("{}", Error_type::File_too_large), "File too large");
        assert_eq!(format!("{}", Error_type::No_attribute), "No attribute");
        assert_eq!(format!("{}", Error_type::Name_too_long), "Name too long");
        assert_eq!(format!("{}", Error_type::Corrupted), "Corrupted");
        assert_eq!(format!("{}", Error_type::No_memory), "No memory");
        assert_eq!(format!("{}", Error_type::No_space_left), "No space left");
        assert_eq!(format!("{}", Error_type::Time_error), "Time error");
        assert_eq!(format!("{}", Error_type::Invalid_inode), "Invalid inode");
        assert_eq!(format!("{}", Error_type::Other), "Other");
    }

    #[test]
    fn Test_error_debug() {
        // Test debug formatting
        let error = Error_type::Permission_denied;
        let debug_str = format!("{:?}", error);
        assert_eq!(debug_str, "Permission_denied");
    }

    #[test]
    fn Test_error_equality() {
        // Test equality and cloning
        let error1 = Error_type::Not_found;
        let error2 = Error_type::Not_found;
        let error3 = Error_type::Permission_denied;

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);

        let cloned = error1.clone();
        assert_eq!(error1, cloned);
    }

    #[test]
    fn Test_error_conversions() {
        // Test conversion to NonZeroU32
        let error = Error_type::Not_found;
        let discriminant: NonZeroU32 = error.into();
        assert_eq!(discriminant.get(), 3);

        // Test explicit discriminant access
        assert_eq!(error.Get_discriminant().get(), 3);
    }

    #[test]
    fn Test_result_type() {
        // Test the Result_type alias
        let success: Result_type<i32> = Ok(42);
        let failure: Result_type<i32> = Err(Error_type::Permission_denied);

        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        assert!(failure.is_err());
        assert_eq!(failure.unwrap_err(), Error_type::Permission_denied);
    }

    #[test]
    fn Test_error_categories() {
        // Test that errors can be categorized by their discriminant ranges

        // Initialization errors (1-3 range roughly)
        assert!(matches!(
            Error_type::Failed_to_initialize_file_system
                .Get_discriminant()
                .get(),
            1
        ));
        assert!(matches!(
            Error_type::Already_initialized.Get_discriminant().get(),
            22
        ));
        assert!(matches!(
            Error_type::Not_initialized.Get_discriminant().get(),
            23
        ));

        // Permission errors
        assert!(matches!(
            Error_type::Permission_denied.Get_discriminant().get(),
            2
        ));
        assert!(matches!(
            Error_type::Invalid_mode.Get_discriminant().get(),
            19
        ));

        // File/Directory errors
        assert!(matches!(Error_type::Not_found.Get_discriminant().get(), 3));
        assert!(matches!(
            Error_type::Already_exists.Get_discriminant().get(),
            4
        ));
        assert!(matches!(
            Error_type::Directory_already_exists
                .Get_discriminant()
                .get(),
            5
        ));
    }

    #[test]
    fn Test_error_copy_semantics() {
        // Test that Error_type implements Copy
        let error = Error_type::File_system_full;
        let copied = error; // This should work due to Copy trait

        // Both should be usable
        assert_eq!(error, Error_type::File_system_full);
        assert_eq!(copied, Error_type::File_system_full);
        assert_eq!(error, copied);
    }

    #[test]
    fn Test_error_size() {
        // Ensure Error_type has a reasonable size for an enum
        use core::mem::size_of;

        // Should be small since it's a C-style enum
        assert!(size_of::<Error_type>() <= 4); // Should be 4 bytes or less
    }

    #[test]
    fn Test_nonzero_conversion() {
        // Test that all errors convert to valid NonZeroU32
        let errors = [
            Error_type::Failed_to_initialize_file_system,
            Error_type::Permission_denied,
            Error_type::Not_found,
            Error_type::Already_exists,
            Error_type::Directory_already_exists,
            Error_type::File_system_full,
            Error_type::File_system_error,
            Error_type::Invalid_path,
            Error_type::Invalid_file,
            Error_type::Invalid_directory,
            Error_type::Invalid_symbolic_link,
            Error_type::Unknown,
            Error_type::Invalid_identifier,
            Error_type::Failed_to_get_task_informations,
            Error_type::Failed_to_get_users_informations,
            Error_type::Too_many_mounted_file_systems,
            Error_type::Too_many_open_files,
            Error_type::Internal_error,
            Error_type::Invalid_mode,
            Error_type::Unsupported_operation,
            Error_type::Ressource_busy,
            Error_type::Already_initialized,
            Error_type::Not_initialized,
            Error_type::Failed_to_get_users_manager_instance,
            Error_type::Failed_to_get_task_manager_instance,
            Error_type::Invalid_parameter,
            Error_type::Invalid_flags,
            Error_type::Not_directory,
            Error_type::Is_directory,
            Error_type::Input_output,
            Error_type::Directory_not_empty,
            Error_type::File_too_large,
            Error_type::No_attribute,
            Error_type::Name_too_long,
            Error_type::Corrupted,
            Error_type::No_memory,
            Error_type::No_space_left,
            Error_type::Time_error,
            Error_type::Invalid_inode,
            Error_type::Other,
        ];

        for error in errors.iter() {
            let discriminant = error.Get_discriminant();
            assert!(discriminant.get() > 0);

            let converted: NonZeroU32 = (*error).into();
            assert_eq!(discriminant, converted);
        }
    }
}
