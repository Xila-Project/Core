use std::ops::{Add, AddAssign};

/// Inode type
/// 
/// This type is used to identify an inode (file, directory, named pipe, etc.) uniquely in a file system.
/// It is a wrapper around a `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Inode_type(u64);

impl Inode_type {
    pub const Minimum: Self = Inode_type(1);

    pub const fn New(Item: u64) -> Self {
        Inode_type(Item)
    }
}

impl From<u64> for Inode_type {
    fn from(item: u64) -> Self {
        Inode_type(item)
    }
}

impl From<Inode_type> for u64 {
    fn from(item: Inode_type) -> Self {
        item.0
    }
}

impl Add<u64> for Inode_type {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        Inode_type(self.0 + other)
    }
}

impl AddAssign<u64> for Inode_type {
    fn add_assign(&mut self, other: u64) {
        self.0 += other;
    }
}
