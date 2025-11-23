use core::{
    fmt::Debug,
    ops::{Add, AddAssign},
};

use task::{TaskIdentifier, TaskIdentifierInner};

use crate::{FileIdentifier, FileIdentifierInner};

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

        let file = self.0 as FileIdentifierInner;
        let file = FileIdentifier::new(file);

        (task, file)
    }

    pub const fn get_file(self) -> FileIdentifier {
        self.split().1
    }

    pub const fn get_minimum(task: TaskIdentifier) -> Self {
        Self::new(task, FileIdentifier::MINIMUM_FILE)
    }

    pub const fn get_maximum(task: TaskIdentifier) -> Self {
        Self::new(task, FileIdentifier::MAXIMUM_FILE)
    }

    pub const fn into_inner(self) -> usize {
        self.0
    }

    pub const fn from_raw(value: usize) -> Self {
        Self(value)
    }
}

impl AddAssign<usize> for UniqueFileIdentifier {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Add<usize> for UniqueFileIdentifier {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl IntoIterator for UniqueFileIdentifier {
    type Item = UniqueFileIdentifier;
    type IntoIter = UniqueFileIdentifierIterator;

    fn into_iter(self) -> Self::IntoIter {
        let (task, _) = self.split();

        UniqueFileIdentifierIterator {
            current: self,
            end: UniqueFileIdentifier::get_maximum(task),
        }
    }
}

pub struct UniqueFileIdentifierIterator {
    current: UniqueFileIdentifier,
    end: UniqueFileIdentifier,
}

impl Iterator for UniqueFileIdentifierIterator {
    type Item = UniqueFileIdentifier;

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
