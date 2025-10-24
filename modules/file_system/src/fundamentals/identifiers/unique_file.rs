use core::fmt::Debug;

use task::TaskIdentifier;

use super::{
    FileIdentifier, FileIdentifierInner, FileSystemIdentifier, FileSystemIdentifierInner,
    LocalFileIdentifier,
};

/// Unique file identifier type
///
/// This type is used to identify an opened file in the virtual file system.
/// It is used for the file identification between the virtual file system and the outside world.
/// It is similar to a file descriptor in Unix-like systems.
/// It is a wrapper around a tuple of [`FileSystemIdentifier`] and [`FileIdentifier`].
/// It is unique from the virtual file system point of view.
///
/// # Example
///
/// ```rust
/// use file_system::{UniqueFileIdentifier, FileIdentifier, FileSystemIdentifier, LocalFileIdentifier};
///
/// use task::TaskIdentifier;
///
/// let Identifier = UniqueFileIdentifier::new(
///     FileSystemIdentifier::from(0x1234),
///     FileIdentifier::from(0x5678),
/// );
///
/// let (File_system, File) = Identifier.split();
///
/// assert_eq!(File_system, FileSystemIdentifier::from(0x1234));
/// assert_eq!(File, FileIdentifier::from(0x5678));
///
/// let (File_system, Local_file) = Identifier.into_local_file_identifier(TaskIdentifier::from(0x9ABC));
///
/// assert_eq!(File_system, FileSystemIdentifier::from(0x1234));
/// assert_eq!(Local_file, LocalFileIdentifier::new(TaskIdentifier::from(0x9ABC), FileIdentifier::from(0x5678)));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct UniqueFileIdentifier(usize);

impl Debug for UniqueFileIdentifier {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (file_system_identifier, file_identifier) = self.split();

        formatter
            .debug_struct("UniqueFileIdentifier")
            .field("File_system_identifier", &file_system_identifier)
            .field("File_identifier", &file_identifier)
            .finish()
    }
}

impl UniqueFileIdentifier {
    const FILE_SYSTEM_IDENTIFIER_POSITION: u8 = FileIdentifier::SIZE_BITS;

    pub const fn new(file_system: FileSystemIdentifier, file: FileIdentifier) -> Self {
        let file_system_identifier = file_system.as_inner();
        let file_identifier = file.into_inner();

        Self(
            (file_system_identifier as usize) << Self::FILE_SYSTEM_IDENTIFIER_POSITION
                | file_identifier as usize,
        )
    }

    pub const fn split(&self) -> (FileSystemIdentifier, FileIdentifier) {
        let file_system = self.0 >> FileIdentifierInner::BITS;
        let file_system = FileSystemIdentifier::new(file_system as FileSystemIdentifierInner);

        let file = self.0 as FileIdentifierInner;
        let file = FileIdentifier::new(file);

        (file_system, file)
    }

    pub const fn into_local_file_identifier(
        self,
        task: TaskIdentifier,
    ) -> (FileSystemIdentifier, LocalFileIdentifier) {
        let (file_system, file) = self.split();

        let local_file = LocalFileIdentifier::new(task, file);

        (file_system, local_file)
    }

    pub const fn into_inner(self) -> usize {
        self.0
    }

    /// This function is shouldn't be used because it doesn't check the validity of the file identifier.
    pub const fn from_raw(inner: usize) -> Self {
        Self(inner)
    }
}

impl From<UniqueFileIdentifier> for usize {
    fn from(identifier: UniqueFileIdentifier) -> Self {
        identifier.0
    }
}

impl From<usize> for UniqueFileIdentifier {
    fn from(identifier: usize) -> Self {
        UniqueFileIdentifier(identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_file_identifier() {
        let identifier = UniqueFileIdentifier::new(
            FileSystemIdentifier::from(0x1234),
            FileIdentifier::from(0x5678),
        );
        assert_eq!(
            identifier.split(),
            (
                FileSystemIdentifier::new(0x1234),
                FileIdentifier::new(0x5678)
            )
        );
    }
}
