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
pub type FileIdentifierInner = u16;
#[cfg(target_pointer_width = "64")]
pub type FileIdentifierInner = u32;

/// Type-safe wrapper for file identifiers.
///
/// File identifiers are used to reference opened files within a task's context,
/// similar to file descriptors in Unix-like systems. Each task maintains its own
/// file identifier space, allowing for task isolation and security.
///
/// # Standard File Identifiers
///
/// The following standard identifiers are predefined:
/// - [`FileIdentifier::STANDARD_IN`] (0) - Standard input
/// - [`FileIdentifier::STANDARD_OUT`] (1) - Standard output  
/// - [`FileIdentifier::STANDARD_ERROR`] (2) - Standard error
/// - [`FileIdentifier::MINIMUM`] (3) - First available identifier for regular files
///
/// # Examples
///
/// ```rust
/// use file_system::FileIdentifier;
///
/// // Standard file identifiers
/// assert_eq!(FileIdentifier::STANDARD_IN.into_inner(), 0);
/// assert_eq!(FileIdentifier::STANDARD_OUT.into_inner(), 1);
/// assert_eq!(FileIdentifier::STANDARD_ERROR.into_inner(), 2);
///
/// // Create a custom file identifier
/// let file_id = FileIdentifier::new(42);
/// assert_eq!(file_id.into_inner(), 42);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct FileIdentifier(FileIdentifierInner);

impl FileIdentifier {
    /// Size in bits of the underlying identifier type.
    pub const SIZE_BITS: u8 = core::mem::size_of::<FileIdentifierInner>() as u8 * 8;

    /// Standard input file identifier (traditionally 0).
    pub const STANDARD_IN: FileIdentifier = FileIdentifier::new(0);

    /// Standard output file identifier (traditionally 1).
    pub const STANDARD_OUT: FileIdentifier = FileIdentifier::new(1);

    /// Standard error file identifier (traditionally 2).
    pub const STANDARD_ERROR: FileIdentifier = FileIdentifier::new(2);

    /// Minimum file identifier available for regular files.
    ///
    /// Regular files should use identifiers starting from this value to avoid
    /// conflicts with internal or reserved identifiers.
    pub const MINIMUM: FileIdentifier = FileIdentifier::new(3);

    /// Maximum possible file identifier value.
    pub const MAXIMUM: FileIdentifier = FileIdentifier::new(FileIdentifierInner::MAX);

    /// Create a new file identifier from a raw value.
    ///
    /// # Arguments
    ///
    /// * `Identifier` - The raw identifier value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_system::FileIdentifier;
    ///
    /// let file_id = FileIdentifier::new(5);
    /// assert_eq!(file_id.into_inner(), 5);
    /// ```
    pub const fn new(identifier: FileIdentifierInner) -> Self {
        Self(identifier)
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
    /// use file_system::FileIdentifier;
    ///
    /// let file_id = FileIdentifier::new(42);
    /// assert_eq!(file_id.into_inner(), 42);
    /// ```
    pub const fn into_inner(self) -> FileIdentifierInner {
        self.0
    }
}

impl From<FileIdentifierInner> for FileIdentifier {
    fn from(internal_file_identifier: FileIdentifierInner) -> Self {
        FileIdentifier(internal_file_identifier)
    }
}

impl From<FileIdentifier> for FileIdentifierInner {
    fn from(internal_file_identifier: FileIdentifier) -> Self {
        internal_file_identifier.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_identifier() {
        let identifier = FileIdentifier::from(0x1234);
        assert_eq!(identifier, FileIdentifier::new(0x1234));
        assert_eq!(FileIdentifierInner::from(identifier), 0x1234);
    }
}
