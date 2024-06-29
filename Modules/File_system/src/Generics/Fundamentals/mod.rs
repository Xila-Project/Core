use std::ops;

mod Flags;
mod Identifiers;
mod Path;
mod Permission;

pub use Flags::*;
pub use Identifiers::*;
pub use Path::*;
pub use Permission::*;

#[derive(Default, PartialOrd, PartialEq, Eq, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Size_type(u64);

impl PartialEq<usize> for Size_type {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other as u64
    }
}

impl From<usize> for Size_type {
    fn from(item: usize) -> Self {
        Size_type(item as u64)
    }
}

impl From<u64> for Size_type {
    fn from(item: u64) -> Self {
        Size_type(item)
    }
}

impl From<Size_type> for usize {
    fn from(item: Size_type) -> Self {
        item.0 as usize
    }
}

impl From<Size_type> for u64 {
    fn from(item: Size_type) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Position_type {
    Start(u64),
    Current(i64),
    End(i64),
}

#[cfg(feature = "std")]
impl From<Position_type> for std::io::SeekFrom {
    fn from(Position: Position_type) -> Self {
        match Position {
            Position_type::Start(Item) => std::io::SeekFrom::Start(Item),
            Position_type::Current(Item) => std::io::SeekFrom::Current(Item),
            Position_type::End(Item) => std::io::SeekFrom::End(Item),
        }
    }
}

#[repr(transparent)]
pub struct Block_type(pub [u8; 512]);

impl Default for Block_type {
    fn default() -> Self {
        Block_type([0; 512])
    }
}

impl ops::Add<Size_type> for Size_type {
    type Output = Size_type;

    fn add(self, rhs: Size_type) -> Self::Output {
        Size_type(self.0 + rhs.0)
    }
}
