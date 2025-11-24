//! File identifier types for opened files.
//!
//! This module provides file identifier types used to identify opened files
//! within a task's file descriptor table, similar to file descriptors in
//! Unix-like systems.

use crate::UniqueFileIdentifier;
use core::num::NonZero;
use task::TaskIdentifier;

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
/// assert_eq!(1u16, FileIdentifier::STANDARD_IN.into());
/// assert_eq!(2u16, FileIdentifier::STANDARD_OUT.into());
/// assert_eq!(3u16, FileIdentifier::STANDARD_ERROR.into());
///
/// // Create a custom file identifier
/// let file_id = FileIdentifier::new(42).unwrap();
/// assert_eq!(42u16, file_id.into());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct FileIdentifier(NonZero<u16>);

impl FileIdentifier {
    /// Size in bits of the underlying identifier type.
    pub const SIZE_BITS: u8 = core::mem::size_of::<FileIdentifierInner>() as u8 * 8;

    const DIRECTORY_FLAG: FileIdentifierInner = 1 << (Self::SIZE_BITS - 1);

    /// Standard input file identifier (traditionally 0).
    pub const STANDARD_IN: Self = Self::new_panic(1);

    /// Standard output file identifier (traditionally 1).
    pub const STANDARD_OUT: Self = Self::new_panic(2);

    /// Standard error file identifier (traditionally 2).
    pub const STANDARD_ERROR: Self = Self::new_panic(3);

    /// Minimum file identifier available for regular files.
    ///
    /// Regular files should use identifiers starting from this value to avoid
    /// conflicts with internal or reserved identifiers.
    pub const MINIMUM_FILE: Self = Self::new_panic(4);

    /// Maximum possible file identifier value.
    pub const MAXIMUM_FILE: Self = Self::new_panic(Self::DIRECTORY_FLAG - 1);

    pub const MINIMUM_DIRECTORY: Self = Self::new_panic(Self::DIRECTORY_FLAG);

    pub const MAXIMUM_DIRECTORY: Self = Self::new_panic(FileIdentifierInner::MAX);
    /// Create a new file identifier from a raw value.
    ///
    /// # Arguments
    ///
    /// * `Identifier` - The raw identifier value.
    pub const fn new(identifier: u16) -> Option<Self> {
        if let Some(non_zero) = NonZero::new(identifier) {
            Some(Self(non_zero))
        } else {
            None
        }
    }

    pub const fn new_panic(identifier: u16) -> Self {
        Self::new(identifier).expect("FileIdentifier cannot be zero")
    }

    /// Get the raw identifier value.
    ///
    /// # Returns
    ///
    /// The underlying raw identifier value.
    pub const fn into_inner(self) -> FileIdentifierInner {
        self.0.get()
    }

    pub const fn into_unique(self, task: TaskIdentifier) -> UniqueFileIdentifier {
        UniqueFileIdentifier::new(task, self)
    }

    pub const fn is_directory(self) -> bool {
        (self.0.get() & Self::DIRECTORY_FLAG) != 0
    }
}

impl TryFrom<FileIdentifierInner> for FileIdentifier {
    type Error = virtual_file_system::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        NonZero::new(value)
            .map(FileIdentifier)
            .ok_or(virtual_file_system::Error::InvalidIdentifier)
    }
}

impl From<FileIdentifier> for u16 {
    fn from(file_identifier: FileIdentifier) -> Self {
        file_identifier.0.get()
    }
}
