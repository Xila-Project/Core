use core::{
    fmt::Debug,
    ops::{Add, AddAssign},
};

use Task::{Task_identifier_inner_type, Task_identifier_type};

use super::{
    File_identifier_inner_type, File_identifier_type, File_system_identifier_type,
    Unique_file_identifier_type,
};

/// Local file type
///
/// This type is used to identify an opened file in a file system.
/// It is used for the file identification between the file system and the virtual file system.
/// It is a wrapper around a tuple of [`Task_identifier_type`] and [`File_identifier_type`].
/// It is unique from the file system point of view.
///
/// # Example
///
/// ```rust
/// use File_system::{Local_file_identifier_type, File_identifier_type, File_system_identifier_type, Unique_file_identifier_type};
///
/// use Task::Task_identifier_type;
///
/// let Identifier = Local_file_identifier_type::New(
///     Task_identifier_type::from(0x1234),
///     File_identifier_type::from(0x5678),
/// );
///
/// let (Task, File) = Identifier.Split();
///
/// assert_eq!(Task, Task_identifier_type::from(0x1234));
/// assert_eq!(File, File_identifier_type::from(0x5678));
///
/// let Minimum = Local_file_identifier_type::Get_minimum(Task);
/// assert_eq!(Minimum, Local_file_identifier_type::New(Task, File_identifier_type::Minimum));
///
/// let Maximum = Local_file_identifier_type::Get_maximum(Task);
/// assert_eq!(Maximum, Local_file_identifier_type::New(Task, File_identifier_type::Maximum));
///
/// let (Task, Unique_file_identifier) = Identifier.Into_unique_file_identifier(File_system_identifier_type::from(0x9ABC));
///
/// assert_eq!(Task, Task_identifier_type::from(0x1234));
/// assert_eq!(Unique_file_identifier, Unique_file_identifier_type::New(File_system_identifier_type::from(0x9ABC), File_identifier_type::from(0x5678)));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Local_file_identifier_type(usize);

impl Local_file_identifier_type {
    const Task_position: u8 = File_identifier_type::Size_bits;

    pub const fn New(Task: Task_identifier_type, File: File_identifier_type) -> Self {
        let Task = Task.Into_inner();
        let File = File.Into_inner();

        Self((Task as usize) << Self::Task_position | File as usize)
    }

    pub const fn Split(self) -> (Task_identifier_type, File_identifier_type) {
        let Task = self.0 >> File_identifier_type::Size_bits;
        let Task = Task_identifier_type::New(Task as Task_identifier_inner_type);

        let File = self.0 as File_identifier_inner_type;
        let File = File_identifier_type::New(File);

        (Task, File)
    }

    pub const fn Get_minimum(Task: Task_identifier_type) -> Self {
        Self::New(Task, File_identifier_type::Minimum)
    }

    pub const fn Get_maximum(Task: Task_identifier_type) -> Self {
        Self::New(Task, File_identifier_type::Maximum)
    }

    pub const fn Into_unique_file_identifier(
        self,
        File_system: File_system_identifier_type,
    ) -> (Task_identifier_type, Unique_file_identifier_type) {
        let (Task, File) = self.Split();

        let Unique_file = Unique_file_identifier_type::New(File_system, File);

        (Task, Unique_file)
    }

    pub const fn Into_inner(self) -> usize {
        self.0
    }
}

impl Debug for Local_file_identifier_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (Task, File) = self.Split();

        Formatter
            .debug_struct("Local_file_identifier_type")
            .field("Task", &Task)
            .field("File", &File)
            .finish()
    }
}

impl AddAssign<usize> for Local_file_identifier_type {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Add<usize> for Local_file_identifier_type {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl IntoIterator for Local_file_identifier_type {
    type Item = Local_file_identifier_type;
    type IntoIter = Local_file_identifier_iterator_type;

    fn into_iter(self) -> Self::IntoIter {
        let (Task, _) = self.Split();

        Local_file_identifier_iterator_type {
            Current: self,
            End: Local_file_identifier_type::Get_maximum(Task),
        }
    }
}

pub struct Local_file_identifier_iterator_type {
    Current: Local_file_identifier_type,
    End: Local_file_identifier_type,
}

impl Iterator for Local_file_identifier_iterator_type {
    type Item = Local_file_identifier_type;

    fn next(&mut self) -> Option<Self::Item> {
        if self.Current < self.End {
            let Current = self.Current;
            self.Current += 1;
            Some(Current)
        } else {
            None
        }
    }
}
