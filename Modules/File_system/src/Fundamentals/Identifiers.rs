use core::ops::{Add, AddAssign};
use std::fmt::Debug;

#[cfg(target_pointer_width = "32")]
pub type File_identifier_inner_type = u16;
#[cfg(target_pointer_width = "64")]
pub type File_identifier_inner_type = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct File_identifier_type(File_identifier_inner_type);

impl File_identifier_type {
    pub const Stdin: File_identifier_type = File_identifier_type::New(0);
    pub const Stdout: File_identifier_type = File_identifier_type::New(1);
    pub const Stderr: File_identifier_type = File_identifier_type::New(2);

    pub const fn New(Identifier: File_identifier_inner_type) -> Self {
        Self(Identifier)
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

#[cfg(target_pointer_width = "32")]
pub type File_system_identifier_inner_type = u16;
#[cfg(target_pointer_width = "64")]
pub type File_system_identifier_inner_type = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct File_system_identifier_type(File_system_identifier_inner_type);

impl File_system_identifier_type {
    pub const fn New(Identifier: File_system_identifier_inner_type) -> Self {
        Self(Identifier)
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Unique_file_identifier_type(usize);

impl Debug for Unique_file_identifier_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (File_system_identifier, File_identifier) = self.Split();

        Formatter
            .debug_struct("Unique_file_identifier_type")
            .field("", &File_system_identifier)
            .field("", &File_identifier)
            .finish()
    }
}

impl Unique_file_identifier_type {
    pub fn New(
        File_system_identifier: File_system_identifier_type,
        File_identifier: File_identifier_type,
    ) -> Self {
        Self(
            (File_system_identifier.0 as usize) << File_identifier_inner_type::BITS
                | File_identifier.0 as usize,
        )
    }

    pub fn Split(self) -> (File_system_identifier_type, File_identifier_type) {
        let File_system = self.0 >> File_identifier_inner_type::BITS;
        let File_system =
            File_system_identifier_type::from(File_system as File_system_identifier_inner_type);

        let File = self.0 as File_identifier_inner_type;
        let File = File_identifier_type::from(File);

        (File_system, File)
    }
}

impl From<Unique_file_identifier_type> for usize {
    fn from(Identifier: Unique_file_identifier_type) -> Self {
        Identifier.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test_file_identifier() {
        let Identifier = File_identifier_type::from(0x1234);
        assert_eq!(Identifier, File_identifier_type::New(0x1234));
        assert_eq!(File_identifier_inner_type::from(Identifier), 0x1234);
    }

    #[test]
    fn Test_file_system_identifier() {
        let Identifier = File_system_identifier_type::from(0x1234);
        assert_eq!(Identifier, File_system_identifier_type::New(0x1234));
        assert_eq!(File_system_identifier_inner_type::from(Identifier), 0x1234);
    }

    #[test]
    fn Test_unique_file_identifier() {
        let Identifier = Unique_file_identifier_type::New(
            File_system_identifier_type::from(0x1234),
            File_identifier_type::from(0x5678),
        );
        assert_eq!(
            Identifier.Split(),
            (
                File_system_identifier_type::New(0x1234),
                File_identifier_type::New(0x5678)
            )
        );
    }
}
