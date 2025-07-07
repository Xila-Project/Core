use core::ops::{Add, AddAssign};

/// File system identifier inner type
///
/// This is the inner/raw type of [`File_system_identifier_type`].
/// It size is the half of the `target_pointer_width` :
/// - 16 bits on 32 bits systems
/// - 32 bits on 64 bits systems
#[cfg(target_pointer_width = "32")]
pub type File_system_identifier_inner_type = u16;
#[cfg(target_pointer_width = "64")]
pub type File_system_identifier_inner_type = u32;

/// File system identifier type
///
/// This type is used to identify a file system in the virtual file system.
/// It is a wrapper around [`File_system_identifier_inner_type`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct File_system_identifier_type(File_system_identifier_inner_type);

impl File_system_identifier_type {
    pub const PIPE_FILE_SYSTEM: File_system_identifier_type = File_system_identifier_type::New(0);
    pub const DEVICE_FILE_SYSTEM: File_system_identifier_type = File_system_identifier_type::New(1);
    pub const NETWORK_SOCKET_FILE_SYSTEM: File_system_identifier_type =
        File_system_identifier_type::New(2);
    pub const LOCAL_SOCKET_FILE_SYSTEM: File_system_identifier_type =
        File_system_identifier_type::New(3);

    pub const MINIMUM: File_system_identifier_type = File_system_identifier_type::New(4);
    pub const MAXIMUM: File_system_identifier_type =
        File_system_identifier_type::New(File_system_identifier_inner_type::MAX);

    pub const fn New(Identifier: File_system_identifier_inner_type) -> Self {
        Self(Identifier)
    }

    pub const fn As_inner(self) -> File_system_identifier_inner_type {
        self.0
    }
}

impl AddAssign<File_system_identifier_inner_type> for File_system_identifier_type {
    fn add_assign(&mut self, rhs: File_system_identifier_inner_type) {
        self.0 += rhs;
    }
}

impl Add<File_system_identifier_inner_type> for File_system_identifier_type {
    type Output = Self;

    fn add(self, rhs: File_system_identifier_inner_type) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl From<File_system_identifier_inner_type> for File_system_identifier_type {
    fn from(Internal_file_system_identifier: File_system_identifier_inner_type) -> Self {
        File_system_identifier_type(Internal_file_system_identifier)
    }
}

impl From<File_system_identifier_type> for File_system_identifier_inner_type {
    fn from(Internal_file_system_identifier: File_system_identifier_type) -> Self {
        Internal_file_system_identifier.0
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_file_system_identifier() {
        let Identifier = File_system_identifier_type::from(0x1234);
        assert_eq!(Identifier, File_system_identifier_type::New(0x1234));
        assert_eq!(File_system_identifier_inner_type::from(Identifier), 0x1234);
    }
}
