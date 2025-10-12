use core::{
    fmt::Debug,
    ops::{Add, AddAssign},
};

use task::{TaskIdentifier, TaskIdentifierInner};

use super::{FileIdentifier, FileIdentifierInner, FileSystemIdentifier, UniqueFileIdentifier};

/// Local file type
///
/// This type is used to identify an opened file in a file system.
/// It is used for the file identification between the file system and the virtual file system.
/// It is a wrapper around a tuple of [`TaskIdentifier`] and [`FileIdentifier`].
/// It is unique from the file system point of view.
///
/// # Example
///
/// ```rust
/// use file_system::{Local_file_identifier_type, File_identifier_type, File_system_identifier_type, Unique_file_identifier_type};
///
/// use task::TaskIdentifier;
///
/// let Identifier = Local_file_identifier_type::new(
///     TaskIdentifier::from(0x1234),
///     File_identifier_type::from(0x5678),
/// );
///
/// let (Task, File) = Identifier.Split();
///
/// assert_eq!(Task, TaskIdentifier::from(0x1234));
/// assert_eq!(File, File_identifier_type::from(0x5678));
///
/// let Minimum = Local_file_identifier_type::get_minimum(Task);
/// assert_eq!(Minimum, Local_file_identifier_type::new(Task, File_identifier_type::Minimum));
///
/// let Maximum = Local_file_identifier_type::get_maximum(Task);
/// assert_eq!(Maximum, Local_file_identifier_type::new(Task, File_identifier_type::Maximum));
///
/// let (Task, Unique_file_identifier) = Identifier.Into_unique_file_identifier(File_system_identifier_type::from(0x9ABC));
///
/// assert_eq!(Task, TaskIdentifier::from(0x1234));
/// assert_eq!(Unique_file_identifier, Unique_file_identifier_type::new(File_system_identifier_type::from(0x9ABC), File_identifier_type::from(0x5678)));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct LocalFileIdentifier(usize);

impl LocalFileIdentifier {
    const TASK_POSITION: u8 = FileIdentifier::SIZE_BITS;

    pub const fn new(task: TaskIdentifier, file: FileIdentifier) -> Self {
        let task = task.into_inner();
        let file = file.into_inner();

        Self((task as usize) << Self::TASK_POSITION | file as usize)
    }

    pub const fn split(self) -> (TaskIdentifier, FileIdentifier) {
        let task = self.0 >> FileIdentifier::SIZE_BITS;
        let task = TaskIdentifier::new(task as TaskIdentifierInner);

        let file = self.0 as FileIdentifierInner;
        let file = FileIdentifier::new(file);

        (task, file)
    }

    pub const fn get_minimum(task: TaskIdentifier) -> Self {
        Self::new(task, FileIdentifier::MINIMUM)
    }

    pub const fn get_maximum(task: TaskIdentifier) -> Self {
        Self::new(task, FileIdentifier::MAXIMUM)
    }

    pub const fn into_unique_file_identifier(
        self,
        file_system: FileSystemIdentifier,
    ) -> (TaskIdentifier, UniqueFileIdentifier) {
        let (task, file) = self.split();

        let unique_file = UniqueFileIdentifier::new(file_system, file);

        (task, unique_file)
    }

    pub const fn into_inner(self) -> usize {
        self.0
    }
}

impl Debug for LocalFileIdentifier {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (task, file) = self.split();

        formatter
            .debug_struct("Local_file_identifier_type")
            .field("Task", &task)
            .field("File", &file)
            .finish()
    }
}

impl AddAssign<usize> for LocalFileIdentifier {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Add<usize> for LocalFileIdentifier {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl IntoIterator for LocalFileIdentifier {
    type Item = LocalFileIdentifier;
    type IntoIter = LocalFileIdentifierIterator;

    fn into_iter(self) -> Self::IntoIter {
        let (task, _) = self.split();

        LocalFileIdentifierIterator {
            current: self,
            end: LocalFileIdentifier::get_maximum(task),
        }
    }
}

pub struct LocalFileIdentifierIterator {
    current: LocalFileIdentifier,
    end: LocalFileIdentifier,
}

impl Iterator for LocalFileIdentifierIterator {
    type Item = LocalFileIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let current = self.current;
            self.current += 1;
            Some(current)
        } else {
            None
        }
    }
}
