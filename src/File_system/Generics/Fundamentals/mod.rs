use std::ops;

pub mod Path;
pub use Path::*;

#[derive(Default, PartialOrd, PartialEq, Eq, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Size_type(pub u64);

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

impl From<u64> for Size_type {
    fn from(item: u64) -> Self {
        Size_type(item)
    }
}

impl From<usize> for Size_type {
    fn from(item: usize) -> Self {
        Size_type(item as u64)
    }
}
