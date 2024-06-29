use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct File_system_identifier_type(u8);

impl File_system_identifier_type {
    pub fn New() -> Self {
        Self(0)
    }

    pub const fn New_from(Identifier: u8) -> Self {
        Self(Identifier)
    }
}

impl AddAssign<u8> for File_system_identifier_type {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs;
    }
}

impl Add<u8> for File_system_identifier_type {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl From<u8> for File_system_identifier_type {
    fn from(Internal_file_system_identifier: u8) -> Self {
        File_system_identifier_type(Internal_file_system_identifier)
    }
}

impl From<File_system_identifier_type> for u8 {
    fn from(Internal_file_system_identifier: File_system_identifier_type) -> Self {
        Internal_file_system_identifier.0
    }
}
