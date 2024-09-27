use std::fmt::Debug;

use Task::Task_identifier_type;

use super::{
    File_identifier_inner_type, File_identifier_type, File_system_identifier_inner_type,
    File_system_identifier_type, Local_file_identifier_type,
};

/// Unique file identifier type
///
/// This type is used to identify an opened file in the virtual file system.
/// It is used for the file identification between the virtual file system and the outside world.
/// It is similar to a file descriptor in Unix-like systems.
/// It is a wrapper around a tuple of [`File_system_identifier_type`] and [`File_identifier_type`].
/// It is unique from the virtual file system point of view.
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
    const File_identifier_position: u8 = 0;
    const File_system_identifier_position: u8 = File_identifier_type::Size_bits as u8;

    pub const fn New(File_system: File_system_identifier_type, File: File_identifier_type) -> Self {
        let File_system_identifier = File_system.Into_inner();
        let File_identifier = File.Into_inner();

        Self(
            (File_system_identifier as usize) << Self::File_system_identifier_position
                | File_identifier as usize,
        )
    }

    pub const fn Split(&self) -> (File_system_identifier_type, File_identifier_type) {
        let File_system = self.0 >> File_identifier_inner_type::BITS;
        let File_system =
            File_system_identifier_type::New(File_system as File_system_identifier_inner_type);

        let File = self.0 as File_identifier_inner_type;
        let File = File_identifier_type::New(File);

        (File_system, File)
    }

    pub const fn Into_local_file_identifier(
        self,
        Task: Task_identifier_type,
    ) -> (File_system_identifier_type, Local_file_identifier_type) {
        let (File_system, File) = self.Split();

        let Local_file = Local_file_identifier_type::New(Task, File);

        (File_system, Local_file)
    }

    pub const fn Into_inner(self) -> usize {
        self.0
    }
}

impl From<Unique_file_identifier_type> for usize {
    fn from(Identifier: Unique_file_identifier_type) -> Self {
        Identifier.0
    }
}

impl From<usize> for Unique_file_identifier_type {
    fn from(Identifier: usize) -> Self {
        Unique_file_identifier_type(Identifier)
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

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
