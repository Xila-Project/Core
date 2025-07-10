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
/// use file_system::Inode_type;
///
/// // Create an inode number
/// let inode = Inode_type::new(42);
/// assert_eq!(inode.As_u64(), 42);
///
/// // Inode numbers can be compared
/// let another_inode = Inode_type::new(43);
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
pub struct Inode(u64);

impl Inode {
    /// The minimum valid inode number.
    ///
    /// Most file systems reserve inode 0 for special purposes, so valid
    /// inode numbers typically start from 1.
    pub const MAXIMUM: Self = Inode(1);

    /// Create a new inode identifier from a u64 value.
    ///
    /// # Arguments
    ///
    /// * `Item` - The inode number
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_system::Inode_type;
    ///
    /// let inode = Inode_type::new(123);
    /// assert_eq!(inode.As_u64(), 123);
    /// ```
    pub const fn new(item: u64) -> Self {
        Inode(item)
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
    /// use file_system::Inode_type;
    ///
    /// let inode = Inode_type::new(456);
    /// assert_eq!(inode.As_u64(), 456);
    /// ```
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for Inode {
    fn from(item: u64) -> Self {
        Inode(item)
    }
}

impl From<Inode> for u64 {
    fn from(item: Inode) -> Self {
        item.0
    }
}

impl Add<u64> for Inode {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        Inode(self.0 + other)
    }
}

impl AddAssign<u64> for Inode {
    fn add_assign(&mut self, other: u64) {
        self.0 += other;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_inode_creation() {
        let inode = Inode::new(42);
        assert_eq!(inode.as_u64(), 42);
    }

    #[test]
    fn test_inode_minimum() {
        assert_eq!(Inode::MAXIMUM.as_u64(), 1);
    }

    #[test]
    fn test_inode_const_operations() {
        // Test that New and As_u64 are const functions
        const INODE: Inode = Inode::new(123);
        const VALUE: u64 = INODE.as_u64();

        assert_eq!(VALUE, 123);
        assert_eq!(INODE.as_u64(), 123);
    }

    #[test]
    fn test_inode_conversions() {
        // From u64
        let inode_from_u64: Inode = 456u64.into();
        assert_eq!(inode_from_u64.as_u64(), 456);

        // To u64
        let as_u64: u64 = inode_from_u64.into();
        assert_eq!(as_u64, 456);
    }

    #[test]
    fn test_inode_comparison() {
        let small = Inode::new(10);
        let large = Inode::new(20);

        assert!(small < large);
        assert!(large > small);
        assert!(small <= large);
        assert!(large >= small);
        assert!(small <= small);
        assert!(large >= large);
        assert_eq!(small, small);
        assert_ne!(small, large);
    }

    #[test]
    fn test_inode_ordering() {
        let mut inodes = [
            Inode::new(100),
            Inode::new(50),
            Inode::new(200),
            Inode::new(25),
        ];

        inodes.sort();

        assert_eq!(inodes[0], Inode::new(25));
        assert_eq!(inodes[1], Inode::new(50));
        assert_eq!(inodes[2], Inode::new(100));
        assert_eq!(inodes[3], Inode::new(200));
    }

    #[test]
    fn test_inode_addition() {
        let inode = Inode::new(100);
        let result = inode + 50;
        assert_eq!(result.as_u64(), 150);
    }

    #[test]
    fn test_inode_add_assign() {
        let mut inode = Inode::new(100);
        inode += 25;
        assert_eq!(inode.as_u64(), 125);
    }

    #[test]
    fn test_inode_debug() {
        let inode = Inode::new(789);
        let debug_str = format!("{inode:?}");
        assert!(debug_str.contains("Inode_type"));
        assert!(debug_str.contains("789"));
    }

    #[test]
    fn test_inode_clone_copy() {
        let original = Inode::new(999);
        let cloned = original;
        let copied = original;

        assert_eq!(original, cloned);
        assert_eq!(original, copied);
        assert_eq!(cloned, copied);

        // Test that we can still use original after copying
        assert_eq!(original.as_u64(), 999);
    }

    #[test]
    fn test_inode_zero() {
        let zero = Inode::new(0);
        assert_eq!(zero.as_u64(), 0);
        assert!(zero < Inode::MAXIMUM);
    }

    #[test]
    fn test_inode_max_value() {
        let max_inode = Inode::new(u64::MAX);
        assert_eq!(max_inode.as_u64(), u64::MAX);
    }

    #[test]
    fn test_inode_arithmetic_edge_cases() {
        // Test addition near max value
        let near_max = Inode::new(u64::MAX - 10);
        let result = near_max + 5;
        assert_eq!(result.as_u64(), u64::MAX - 5);
    }

    #[test]
    fn test_inode_type_safety() {
        // Verify that Inode_type is a zero-cost abstraction
        use core::mem::{align_of, size_of};

        assert_eq!(size_of::<Inode>(), size_of::<u64>());
        assert_eq!(align_of::<Inode>(), align_of::<u64>());
    }

    #[test]
    fn test_inode_sequence_operations() {
        let start = Inode::new(1000);
        let mut current = start;

        // Test sequential addition
        for i in 1..=10 {
            current += 1;
            assert_eq!(current.as_u64(), 1000 + i);
        }
    }

    #[test]
    fn test_inode_round_trip_conversion() {
        let original_value = 12345u64;
        let inode = Inode::new(original_value);
        let converted: u64 = inode.into();
        let back_to_inode: Inode = converted.into();

        assert_eq!(original_value, converted);
        assert_eq!(inode, back_to_inode);
    }

    #[test]
    fn test_inode_minimum_comparison() {
        let minimum = Inode::MAXIMUM;
        let zero = Inode::new(0);
        let two = Inode::new(2);

        assert!(zero < minimum);
        assert!(minimum < two);
        assert_eq!(minimum.as_u64(), 1);
    }

    #[test]
    fn test_inode_large_additions() {
        let inode = Inode::new(1000);
        let large_add = 1_000_000u64;
        let result = inode + large_add;

        assert_eq!(result.as_u64(), 1_001_000);
    }

    #[test]
    fn test_inode_multiple_operations() {
        let mut inode = Inode::new(100);

        inode += 10;
        inode += 20;
        inode += 30;

        assert_eq!(inode.as_u64(), 160);

        let added = inode + 40;
        assert_eq!(added.as_u64(), 200);
        assert_eq!(inode.as_u64(), 160); // Original should be unchanged
    }
}
