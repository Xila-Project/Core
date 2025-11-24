use crate::FileIdentifier;
use core::fmt::Debug;
use task::{TaskIdentifier, TaskIdentifierInner};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct UniqueFileIdentifier(usize);

impl Debug for UniqueFileIdentifier {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (task, file) = self.split();

        formatter
            .debug_struct("LocalFileIdentifier")
            .field("task", &task)
            .field("file", &file)
            .finish()
    }
}

impl UniqueFileIdentifier {
    const TASK_POSITION: u8 = FileIdentifier::SIZE_BITS;
    pub const INVALID_FILE_IDENTIFIER: Self = Self(usize::MAX);

    pub const fn new(task: TaskIdentifier, file: FileIdentifier) -> Self {
        let task = task.into_inner();
        let file = file.into_inner();

        Self((task as usize) << Self::TASK_POSITION | file as usize)
    }

    pub const fn split(self) -> (TaskIdentifier, FileIdentifier) {
        let task = self.0 >> FileIdentifier::SIZE_BITS;
        let task = TaskIdentifier::new(task as TaskIdentifierInner);

        let file = FileIdentifier::new_panic(self.0 as _);

        (task, file)
    }

    pub const fn get_file(self) -> FileIdentifier {
        self.split().1
    }

    pub const fn into_inner(self) -> usize {
        self.0
    }

    pub const fn from_raw(value: usize) -> Self {
        Self(value)
    }
}
