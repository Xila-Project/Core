use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct File_identifier_type(u16);

impl From<u16> for File_identifier_type {
    fn from(Internal_file_identifier: u16) -> Self {
        File_identifier_type(Internal_file_identifier)
    }
}

impl From<File_identifier_type> for u16 {
    fn from(Internal_file_identifier: File_identifier_type) -> Self {
        Internal_file_identifier.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct File_system_identifier_type(u16);

impl File_system_identifier_type {
    pub const fn New(Identifier: u16) -> Self {
        Self(Identifier)
    }
}

impl AddAssign<u16> for File_system_identifier_type {
    fn add_assign(&mut self, rhs: u16) {
        self.0 += rhs;
    }
}

impl Add<u16> for File_system_identifier_type {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl From<u16> for File_system_identifier_type {
    fn from(Internal_file_system_identifier: u16) -> Self {
        File_system_identifier_type(Internal_file_system_identifier)
    }
}

impl From<File_system_identifier_type> for u16 {
    fn from(Internal_file_system_identifier: File_system_identifier_type) -> Self {
        Internal_file_system_identifier.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Unique_file_identifier_type(u32);

impl Unique_file_identifier_type {
    pub fn New(
        File_system_identifier: File_system_identifier_type,
        File_identifier: File_identifier_type,
    ) -> Self {
        Self((File_system_identifier.0 as u32) << 16 | File_identifier.0 as u32)
    }

    pub fn Split(self) -> (File_system_identifier_type, File_identifier_type) {
        let File_system_identifier = File_system_identifier_type::New((self.0 >> 16) as u16);
        let File_identifier = File_identifier_type((self.0 & 0xFFFF) as u16);
        (File_system_identifier, File_identifier)
    }
}
