/// File identifier inner type
///
/// This is the inner/raw type of [`File_identifier_type`].
/// It size is the half of the `target_pointer_width` :
/// - 16 bits on 32 bits systems
/// - 32 bits on 64 bits systems
#[cfg(target_pointer_width = "32")]
pub type File_identifier_inner_type = u16;
#[cfg(target_pointer_width = "64")]
pub type File_identifier_inner_type = u32;

/// File identifier type
///
/// This type is used to identify an opened file in a file system.
/// This is similar to a file descriptor in Unix-like systems.
/// It is a wrapper around [`File_identifier_inner_type`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct File_identifier_type(File_identifier_inner_type);

impl File_identifier_type {
    pub const Size_bits: u8 = core::mem::size_of::<File_identifier_inner_type>() as u8 * 8;

    pub const Stdin: File_identifier_type = File_identifier_type::New(0);
    pub const Stdout: File_identifier_type = File_identifier_type::New(1);
    pub const Stderr: File_identifier_type = File_identifier_type::New(2);

    pub const Minimum: File_identifier_type = File_identifier_type::New(3);
    pub const Maximum: File_identifier_type =
        File_identifier_type::New(File_identifier_inner_type::MAX);

    pub const fn New(Identifier: File_identifier_inner_type) -> Self {
        Self(Identifier)
    }

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
