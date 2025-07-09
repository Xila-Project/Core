use core::fmt::Debug;

use task::Task_identifier_type;

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
///
/// # Example
///
/// ```rust
/// use file_system::{Unique_file_identifier_type, File_identifier_type, File_system_identifier_type, Local_file_identifier_type};
///
/// use task::Task_identifier_type;
///
/// let Identifier = Unique_file_identifier_type::New(
///     File_system_identifier_type::from(0x1234),
///     File_identifier_type::from(0x5678),
/// );
///
/// let (File_system, File) = Identifier.Split();
///
/// assert_eq!(File_system, File_system_identifier_type::from(0x1234));
/// assert_eq!(File, File_identifier_type::from(0x5678));
///
/// let (File_system, Local_file) = Identifier.Into_local_file_identifier(Task_identifier_type::from(0x9ABC));
///
/// assert_eq!(File_system, File_system_identifier_type::from(0x1234));
/// assert_eq!(Local_file, Local_file_identifier_type::New(Task_identifier_type::from(0x9ABC), File_identifier_type::from(0x5678)));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Unique_file_identifier_type(usize);

impl Debug for Unique_file_identifier_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (file_system_identifier, file_identifier) = self.split();

        formatter
            .debug_struct("Unique_file_identifier_type")
            .field("File_system_identifier", &file_system_identifier)
            .field("File_identifier", &file_identifier)
            .finish()
    }
}

impl Unique_file_identifier_type {
    const FILE_SYSTEM_IDENTIFIER_POSITION: u8 = File_identifier_type::SIZE_BITS;

    pub const fn new(file_system: File_system_identifier_type, file: File_identifier_type) -> Self {
        let file_system_identifier = file_system.as_inner();
        let file_identifier = file.into_inner();

        Self(
            (file_system_identifier as usize) << Self::FILE_SYSTEM_IDENTIFIER_POSITION
                | file_identifier as usize,
        )
    }

    pub const fn split(&self) -> (File_system_identifier_type, File_identifier_type) {
        let file_system = self.0 >> File_identifier_inner_type::BITS;
        let file_system =
            File_system_identifier_type::new(file_system as File_system_identifier_inner_type);

        let file = self.0 as File_identifier_inner_type;
        let file = File_identifier_type::new(file);

        (file_system, file)
    }

    pub const fn into_local_file_identifier(
        self,
        task: Task_identifier_type,
    ) -> (File_system_identifier_type, Local_file_identifier_type) {
        let (file_system, file) = self.split();

        let local_file = Local_file_identifier_type::new(task, file);

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

impl From<Unique_file_identifier_type> for usize {
    fn from(identifier: Unique_file_identifier_type) -> Self {
        identifier.0
    }
}

impl From<usize> for Unique_file_identifier_type {
    fn from(identifier: usize) -> Self {
        Unique_file_identifier_type(identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_file_identifier() {
        let identifier = Unique_file_identifier_type::new(
            File_system_identifier_type::from(0x1234),
            File_identifier_type::from(0x5678),
        );
        assert_eq!(
            identifier.split(),
            (
                File_system_identifier_type::new(0x1234),
                File_identifier_type::new(0x5678)
            )
        );
    }
}
