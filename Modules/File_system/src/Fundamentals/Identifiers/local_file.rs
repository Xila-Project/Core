use core::{
    fmt::Debug,
    ops::{Add, AddAssign},
};

use task::{Task_identifier_inner_type, Task_identifier_type};

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
/// use file_system::{Local_file_identifier_type, File_identifier_type, File_system_identifier_type, Unique_file_identifier_type};
///
/// use task::Task_identifier_type;
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
/// let Minimum = Local_file_identifier_type::get_minimum(Task);
/// assert_eq!(Minimum, Local_file_identifier_type::New(Task, File_identifier_type::Minimum));
///
/// let Maximum = Local_file_identifier_type::get_maximum(Task);
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
    const TASK_POSITION: u8 = File_identifier_type::SIZE_BITS;

    pub const fn New(Task: Task_identifier_type, File: File_identifier_type) -> Self {
        let task = Task.Into_inner();
        let file = File.Into_inner();

        Self((task as usize) << Self::TASK_POSITION | file as usize)
    }

    pub const fn Split(self) -> (Task_identifier_type, File_identifier_type) {
        let task = self.0 >> File_identifier_type::SIZE_BITS;
        let task = Task_identifier_type::new(task as Task_identifier_inner_type);

        let File = self.0 as File_identifier_inner_type;
        let file = File_identifier_type::New(File);

        (task, file)
    }

    pub const fn get_minimum(Task: Task_identifier_type) -> Self {
        Self::New(Task, File_identifier_type::MINIMUM)
    }

    pub const fn get_maximum(Task: Task_identifier_type) -> Self {
        Self::New(Task, File_identifier_type::MAXIMUM)
    }

    pub const fn Into_unique_file_identifier(
        self,
        file_system: File_system_identifier_type,
    ) -> (Task_identifier_type, Unique_file_identifier_type) {
        let (task, file) = self.Split();

        let Unique_file = Unique_file_identifier_type::New(file_system, file);

        (task, Unique_file)
    }

    pub const fn Into_inner(self) -> usize {
        self.0
    }
}

impl Debug for Local_file_identifier_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (task, file) = self.Split();

        formatter
            .debug_struct("Local_file_identifier_type")
            .field("Task", &task)
            .field("File", &file)
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
        let (task, _) = self.Split();

        Local_file_identifier_iterator_type {
            current: self,
            end: Local_file_identifier_type::get_maximum(task),
        }
    }
}

pub struct Local_file_identifier_iterator_type {
    current: Local_file_identifier_type,
    end: Local_file_identifier_type,
}

impl Iterator for Local_file_identifier_iterator_type {
    type Item = Local_file_identifier_type;

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
