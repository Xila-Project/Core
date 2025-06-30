//! File identifier types for opened files.
//!
//! This module provides file identifier types used to identify opened files
//! within a task's file descriptor table, similar to file descriptors in
//! Unix-like systems.

/// File identifier inner type.
///
/// This is the underlying numeric type used for file identifiers. The size
/// is architecture-dependent to optimize memory usage while providing sufficient
/// identifier space:
/// - 16 bits (0-65,535) on 32-bit systems  
/// - 32 bits (0-4,294,967,295) on 64-bit systems
///
/// This provides a good balance between memory efficiency and identifier space.
#[cfg(target_pointer_width = "32")]
pub type File_identifier_inner_type = u16;
#[cfg(target_pointer_width = "64")]
pub type File_identifier_inner_type = u32;

/// Type-safe wrapper for file identifiers.
///
/// File identifiers are used to reference opened files within a task's context,
/// similar to file descriptors in Unix-like systems. Each task maintains its own
/// file identifier space, allowing for task isolation and security.
///
/// # Standard File Identifiers
///
/// The following standard identifiers are predefined:
/// - [`Standard_in`] (0) - Standard input
/// - [`Standard_out`] (1) - Standard output  
/// - [`Standard_error`] (2) - Standard error
/// - [`Minimum`] (3) - First available identifier for regular files
///
/// # Examples
///
/// ```rust
/// use File_system::File_identifier_type;
///
/// // Standard file identifiers
/// assert_eq!(File_identifier_type::Standard_in.Into_inner(), 0);
/// assert_eq!(File_identifier_type::Standard_out.Into_inner(), 1);
/// assert_eq!(File_identifier_type::Standard_error.Into_inner(), 2);
///
/// // Create a custom file identifier
/// let file_id = File_identifier_type::New(42);
/// assert_eq!(file_id.Into_inner(), 42);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct File_identifier_type(File_identifier_inner_type);

impl File_identifier_type {
    /// Size in bits of the underlying identifier type.
    pub const Size_bits: u8 = core::mem::size_of::<File_identifier_inner_type>() as u8 * 8;

    /// Standard input file identifier (traditionally 0).
    pub const Standard_in: File_identifier_type = File_identifier_type::New(0);

    /// Standard output file identifier (traditionally 1).
    pub const Standard_out: File_identifier_type = File_identifier_type::New(1);

    /// Standard error file identifier (traditionally 2).
    pub const Standard_error: File_identifier_type = File_identifier_type::New(2);

    /// Minimum file identifier available for regular files.
    ///
    /// Regular files should use identifiers starting from this value to avoid
    /// conflicts with standard file identifiers.
    pub const Minimum: File_identifier_type = File_identifier_type::New(3);

    /// Maximum possible file identifier value.
    pub const Maximum: File_identifier_type =
        File_identifier_type::New(File_identifier_inner_type::MAX);

    /// Create a new file identifier from a raw value.
    ///
    /// # Arguments
    ///
    /// * `Identifier` - The raw identifier value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use File_system::File_identifier_type;
    ///
    /// let file_id = File_identifier_type::New(5);
    /// assert_eq!(file_id.Into_inner(), 5);
    /// ```
    pub const fn New(Identifier: File_identifier_inner_type) -> Self {
        Self(Identifier)
    }

    /// Get the raw identifier value.
    ///
    /// # Returns
    ///
    /// The underlying raw identifier value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use File_system::File_identifier_type;
    ///
    /// let file_id = File_identifier_type::New(42);
    /// assert_eq!(file_id.Into_inner(), 42);
    /// ```
    pub const fn Into_inner(self) -> File_identifier_inner_type {
        self.0
    }
}

impl From<File_identifier_inner_type> for File_identifier_type {
    fn from(Internal_file_identifier: File_identifier_inner_type) -> Self {
        File_identifier_type(Internal_file_identifier)
    }
}

impl From<File_identifier_type> for File_identifier_inner_type {
    fn from(Internal_file_identifier: File_identifier_type) -> Self {
        Internal_file_identifier.0
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_file_identifier() {
        let Identifier = File_identifier_type::from(0x1234);
        assert_eq!(Identifier, File_identifier_type::New(0x1234));
        assert_eq!(File_identifier_inner_type::from(Identifier), 0x1234);
    }
}
