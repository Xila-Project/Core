//! File identifier types for opened files.
//!
//! This module provides file identifier types used to identify opened files
//! within a task's file descriptor table, similar to file descriptors in
//! Unix-like systems.

use task::TaskIdentifier;

use crate::UniqueFileIdentifier;

pub type FileIdentifierInner = u16;

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
/// use abi_context::FileIdentifier;
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

    pub const DIRECTORY_FLAG: FileIdentifierInner = 1 << (Self::SIZE_BITS - 1);

    /// Standard input file identifier (traditionally 0).
    pub const STANDARD_IN: Self = Self::new(0);

    /// Standard output file identifier (traditionally 1).
    pub const STANDARD_OUT: Self = Self::new(1);

    /// Standard error file identifier (traditionally 2).
    pub const STANDARD_ERROR: Self = Self::new(2);

    /// Minimum file identifier available for regular files.
    ///
    /// Regular files should use identifiers starting from this value to avoid
    /// conflicts with internal or reserved identifiers.
    pub const MINIMUM_FILE: Self = Self::new(3);

    /// Maximum possible file identifier value.
    pub const MAXIMUM_FILE: Self = Self::new(Self::DIRECTORY_FLAG - 1);

    pub const MINIMUM_DIRECTORY: Self = Self::new(Self::DIRECTORY_FLAG);

    pub const MAXIMUM_DIRECTORY: Self = Self::new(FileIdentifierInner::MAX - 1);

    pub const INVALID: Self = Self::new(FileIdentifierInner::MAX);

    /// Create a new file identifier from a raw value.
    ///
    /// # Arguments
    ///
    /// * `Identifier` - The raw identifier value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use abi_context::FileIdentifier;
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
    /// use abi_context::FileIdentifier;
    ///
    /// let file_id = FileIdentifier::new(42);
    /// assert_eq!(file_id.into_inner(), 42);
    /// ```
    pub const fn into_inner(self) -> FileIdentifierInner {
        self.0
    }

    pub const fn into_unique(self, task: TaskIdentifier) -> UniqueFileIdentifier {
        UniqueFileIdentifier::new(task, self)
    }

    pub const fn is_directory(self) -> bool {
        (self.0 & Self::DIRECTORY_FLAG) != 0
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
