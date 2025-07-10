use core::ops::{Add, AddAssign};

/// File system identifier inner type
///
/// This is the inner/raw type of [`File_system_identifier_type`].
/// It size is the half of the `target_pointer_width` :
/// - 16 bits on 32 bits systems
/// - 32 bits on 64 bits systems
#[cfg(target_pointer_width = "32")]
pub type FileSystemIdentifierInner = u16;
#[cfg(target_pointer_width = "64")]
pub type FileSystemIdentifierInner = u32;

/// File system identifier type
///
/// This type is used to identify a file system in the virtual file system.
/// It is a wrapper around [`File_system_identifier_inner_type`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct FileSystemIdentifier(FileSystemIdentifierInner);

impl FileSystemIdentifier {
    pub const PIPE_FILE_SYSTEM: FileSystemIdentifier = FileSystemIdentifier::new(0);
    pub const DEVICE_FILE_SYSTEM: FileSystemIdentifier = FileSystemIdentifier::new(1);
    pub const NETWORK_SOCKET_FILE_SYSTEM: FileSystemIdentifier = FileSystemIdentifier::new(2);
    pub const LOCAL_SOCKET_FILE_SYSTEM: FileSystemIdentifier = FileSystemIdentifier::new(3);

    pub const MINIMUM: FileSystemIdentifier = FileSystemIdentifier::new(4);
    pub const MAXIMUM: FileSystemIdentifier =
        FileSystemIdentifier::new(FileSystemIdentifierInner::MAX);

    pub const fn new(identifier: FileSystemIdentifierInner) -> Self {
        Self(identifier)
    }

    pub const fn as_inner(self) -> FileSystemIdentifierInner {
        self.0
    }
}

impl AddAssign<FileSystemIdentifierInner> for FileSystemIdentifier {
    fn add_assign(&mut self, rhs: FileSystemIdentifierInner) {
        self.0 += rhs;
    }
}

impl Add<FileSystemIdentifierInner> for FileSystemIdentifier {
    type Output = Self;

    fn add(self, rhs: FileSystemIdentifierInner) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl From<FileSystemIdentifierInner> for FileSystemIdentifier {
    fn from(internal_file_system_identifier: FileSystemIdentifierInner) -> Self {
        FileSystemIdentifier(internal_file_system_identifier)
    }
}

impl From<FileSystemIdentifier> for FileSystemIdentifierInner {
    fn from(internal_file_system_identifier: FileSystemIdentifier) -> Self {
        internal_file_system_identifier.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_system_identifier() {
        let identifier = FileSystemIdentifier::from(0x1234);
        assert_eq!(identifier, FileSystemIdentifier::new(0x1234));
        assert_eq!(FileSystemIdentifierInner::from(identifier), 0x1234);
    }
}
