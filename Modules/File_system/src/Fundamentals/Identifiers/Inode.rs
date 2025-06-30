//! Inode identifier type for file system objects.
//!
//! This module provides the [`Inode_type`] which represents inode numbers
//! used to uniquely identify file system objects within a specific file system.

use core::ops::{Add, AddAssign};

/// Type-safe wrapper for inode numbers.
///
/// An inode (index node) is a unique identifier for file system objects such as
/// files, directories, symbolic links, and other entities within a file system.
/// Each file system object has a unique inode number that persists for the
/// lifetime of the object.
///
/// # Examples
///
/// ```rust
/// use File_system::Inode_type;
///
/// // Create an inode number
/// let inode = Inode_type::New(42);
/// assert_eq!(inode.As_u64(), 42);
///
/// // Inode numbers can be compared
/// let another_inode = Inode_type::New(43);
/// assert!(inode < another_inode);
///
/// // Arithmetic operations are supported
/// let incremented = inode + 10;
/// assert_eq!(incremented.As_u64(), 52);
/// ```
///
/// # Note
///
/// Inode 0 is typically reserved in most file systems. The minimum valid
/// inode number is provided as [`Inode_type::Minimum`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Inode_type(u64);

impl Inode_type {
    /// The minimum valid inode number.
    ///
    /// Most file systems reserve inode 0 for special purposes, so valid
    /// inode numbers typically start from 1.
    pub const Minimum: Self = Inode_type(1);

    /// Create a new inode identifier from a u64 value.
    ///
    /// # Arguments
    ///
    /// * `Item` - The inode number
    ///
    /// # Examples
    ///
    /// ```rust
    /// use File_system::Inode_type;
    ///
    /// let inode = Inode_type::New(123);
    /// assert_eq!(inode.As_u64(), 123);
    /// ```
    pub const fn New(Item: u64) -> Self {
        Inode_type(Item)
    }

    /// Get the inode number as a u64.
    ///
    /// # Returns
    ///
    /// The underlying inode number as a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use File_system::Inode_type;
    ///
    /// let inode = Inode_type::New(456);
    /// assert_eq!(inode.As_u64(), 456);
    /// ```
    pub const fn As_u64(self) -> u64 {
        self.0
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
