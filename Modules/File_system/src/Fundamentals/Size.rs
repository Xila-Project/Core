use std::ops::{Add, AddAssign};

/// Size type
///
/// This type is used to represent the size of data which can be hold by a file system.
/// Since the size of a file system can be very large, this type is a 64-bit unsigned integer.
///
/// # Examples
///
/// ```rust
/// use File_system::Size_type;
///
/// let Size = Size_type::New(0);
/// ```
#[derive(Default, PartialOrd, PartialEq, Eq, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Size_type(u64);

impl Size_type {
    pub const fn New(Item: u64) -> Self {
        Size_type(Item)
    }
}

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

impl Add<Size_type> for Size_type {
    type Output = Size_type;

    fn add(self, rhs: Size_type) -> Self::Output {
        Size_type(self.0 + rhs.0)
    }
}

impl Add<usize> for Size_type {
    type Output = Size_type;

    fn add(self, rhs: usize) -> Self::Output {
        Size_type(self.0 + rhs as u64)
    }
}

impl Add<u64> for Size_type {
    type Output = Size_type;

    fn add(self, rhs: u64) -> Self::Output {
        Size_type(self.0 + rhs)
    }
}

impl Add<Size_type> for usize {
    type Output = Size_type;

    fn add(self, rhs: Size_type) -> Self::Output {
        Size_type(self as u64 + rhs.0)
    }
}

impl Add<Size_type> for u64 {
    type Output = Size_type;

    fn add(self, rhs: Size_type) -> Self::Output {
        Size_type(self + rhs.0)
    }
}

impl AddAssign<Size_type> for Size_type {
    fn add_assign(&mut self, rhs: Size_type) {
        self.0 += rhs.0;
    }
}

impl AddAssign<usize> for Size_type {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs as u64;
    }
}

impl AddAssign<u64> for Size_type {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl AddAssign<Size_type> for usize {
    fn add_assign(&mut self, rhs: Size_type) {
        *self += rhs.0 as usize;
    }
}
