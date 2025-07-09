//! Size representation for file system operations.
//!
//! This module provides the [`Size_type`] wrapper around `u64` for representing
//! sizes, lengths, and byte counts in file system operations. It provides type
//! safety and consistent handling of size values throughout the file system.

use core::{
    fmt::{self, Display, Formatter},
    ops::{Add, AddAssign},
};

/// Type-safe wrapper for size values in file system operations.
///
/// `Size_type` represents sizes, lengths, and byte counts as a 64-bit unsigned integer.
/// This provides a range of 0 to approximately 18 exabytes, which is sufficient for
/// any practical file system operation. The type provides various conversion methods
/// and arithmetic operations for convenient size manipulation.
///
/// # Examples
///
/// ```rust
/// use file_system::Size_type;
///
/// // Create a size representing 1024 bytes
/// let size = Size_type::New(1024);
/// assert_eq!(size.As_u64(), 1024);
///
/// // Convert from usize
/// let size_from_usize: Size_type = 512usize.into();
/// assert_eq!(size_from_usize.As_u64(), 512);
///
/// // Arithmetic operations
/// let total = size + size_from_usize;
/// assert_eq!(total.As_u64(), 1536);
/// ```
///
/// # Type Safety
///
/// Using `Size_type` instead of raw integers helps prevent mixing up different
/// numeric types and provides clearer API signatures throughout the file system.
#[derive(Default, PartialOrd, PartialEq, Eq, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Size_type(u64);

impl Display for Size_type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Size_type {
    /// Create a new size value from a u64.
    ///
    /// # Arguments
    ///
    /// * `Item` - The size value in bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_system::Size_type;
    ///
    /// let size = Size_type::New(2048);
    /// assert_eq!(size.As_u64(), 2048);
    /// ```
    pub const fn New(Item: u64) -> Self {
        Size_type(Item)
    }

    /// Get the size value as a u64.
    ///
    /// # Returns
    ///
    /// The size value in bytes as a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_system::Size_type;
    ///
    /// let size = Size_type::New(4096);
    /// assert_eq!(size.As_u64(), 4096);
    /// ```
    pub const fn As_u64(&self) -> u64 {
        self.0
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_size_creation() {
        let size = Size_type::New(1024);
        assert_eq!(size.As_u64(), 1024);
    }

    #[test]
    fn test_size_default() {
        let size = Size_type::default();
        assert_eq!(size.As_u64(), 0);
    }

    #[test]
    fn test_size_conversions() {
        // From usize
        let size_from_usize: Size_type = 512usize.into();
        assert_eq!(size_from_usize.As_u64(), 512);

        // From u64
        let size_from_u64: Size_type = 1024u64.into();
        assert_eq!(size_from_u64.As_u64(), 1024);

        // To usize
        let as_usize: usize = size_from_u64.into();
        assert_eq!(as_usize, 1024);

        // To u64
        let as_u64: u64 = size_from_u64.into();
        assert_eq!(as_u64, 1024);
    }

    #[test]
    fn test_size_equality() {
        let size1 = Size_type::New(100);
        let size2 = Size_type::New(100);
        let size3 = Size_type::New(200);

        assert_eq!(size1, size2);
        assert_ne!(size1, size3);

        // Test equality with usize
        assert_eq!(size1, 100usize);
        assert_ne!(size1, 200usize);
    }

    #[test]
    fn test_size_comparison() {
        let small = Size_type::New(100);
        let large = Size_type::New(200);

        assert!(small < large);
        assert!(large > small);
        assert!(small <= large);
        assert!(large >= small);
        assert!(small <= small);
        assert!(large >= large);
    }

    #[test]
    fn test_size_addition_with_size() {
        let size1 = Size_type::New(100);
        let size2 = Size_type::New(200);
        let result = size1 + size2;
        assert_eq!(result.As_u64(), 300);
    }

    #[test]
    fn test_size_addition_with_usize() {
        let size = Size_type::New(100);
        let result = size + 50usize;
        assert_eq!(result.As_u64(), 150);

        // Test commutative property
        let result2 = 50usize + size;
        assert_eq!(result2.As_u64(), 150);
    }

    #[test]
    fn test_size_addition_with_u64() {
        let size = Size_type::New(100);
        let result = size + 75u64;
        assert_eq!(result.As_u64(), 175);

        // Test commutative property
        let result2 = 75u64 + size;
        assert_eq!(result2.As_u64(), 175);
    }

    #[test]
    fn test_size_add_assign_with_size() {
        let mut size = Size_type::New(100);
        let other = Size_type::New(50);
        size += other;
        assert_eq!(size.As_u64(), 150);
    }

    #[test]
    fn test_size_add_assign_with_usize() {
        let mut size = Size_type::New(100);
        size += 25usize;
        assert_eq!(size.As_u64(), 125);

        // Test adding to usize
        let mut value = 100usize;
        value += Size_type::New(25);
        assert_eq!(value, 125);
    }

    #[test]
    fn test_size_add_assign_with_u64() {
        let mut size = Size_type::New(100);
        size += 30u64;
        assert_eq!(size.As_u64(), 130);
    }

    #[test]
    fn test_size_display() {
        let size = Size_type::New(12345);
        let display_str = format!("{size}");
        assert_eq!(display_str, "12345");
    }

    #[test]
    fn test_size_debug() {
        let size = Size_type::New(67890);
        let debug_str = format!("{size:?}");
        assert_eq!(debug_str, "Size_type(67890)");
    }

    #[test]
    fn test_size_clone_copy() {
        let original = Size_type::New(999);
        let cloned = original;
        let copied = original;

        assert_eq!(original, cloned);
        assert_eq!(original, copied);
        assert_eq!(cloned, copied);

        // Test that we can still use original after copying
        assert_eq!(original.As_u64(), 999);
    }

    #[test]
    fn test_size_zero() {
        let zero = Size_type::New(0);
        assert_eq!(zero.As_u64(), 0);
        assert_eq!(zero, 0usize);
        assert_eq!(zero, Size_type::default());
    }

    #[test]
    fn test_size_max_value() {
        let max_size = Size_type::New(u64::MAX);
        assert_eq!(max_size.As_u64(), u64::MAX);
    }

    #[test]
    fn test_size_arithmetic_overflow_safety() {
        // Test large values that might overflow in some operations
        let large1 = Size_type::New(u64::MAX / 2);
        let large2 = Size_type::New(u64::MAX / 2);

        // This would overflow, but we're testing the types work correctly
        // In practice, overflow behavior depends on debug/release mode
        let _ = large1 + large2; // Should wrap around in release mode
    }

    #[test]
    fn test_size_type_safety() {
        // Verify that Size_type is a zero-cost abstraction
        use core::mem::{align_of, size_of};

        assert_eq!(size_of::<Size_type>(), size_of::<u64>());
        assert_eq!(align_of::<Size_type>(), align_of::<u64>());
    }

    #[test]
    fn test_size_const_operations() {
        // Test that New and As_u64 are const functions
        const SIZE: Size_type = Size_type::New(42);
        const VALUE: u64 = SIZE.As_u64();

        assert_eq!(VALUE, 42);
        assert_eq!(SIZE.As_u64(), 42);
    }

    #[test]
    fn test_size_mixed_arithmetic() {
        let size = Size_type::New(100);

        // Chain multiple additions
        let result = size + 50usize + 25u64 + Size_type::New(10);
        assert_eq!(result.As_u64(), 185);
    }

    #[test]
    fn test_size_compound_assignments() {
        let mut size = Size_type::New(10);

        size += 5usize;
        size += 3u64;
        size += Size_type::New(2);

        assert_eq!(size.As_u64(), 20);
    }

    #[test]
    fn test_size_comparison_edge_cases() {
        let zero = Size_type::New(0);
        let one = Size_type::New(1);
        let max = Size_type::New(u64::MAX);

        assert!(zero < one);
        assert!(one < max);
        assert!(zero < max);

        assert!(max > one);
        assert!(one > zero);
        assert!(max > zero);
    }

    #[test]
    fn test_size_conversion_edge_cases() {
        // Test conversion from max usize
        let max_usize_as_size: Size_type = usize::MAX.into();
        let back_to_usize: usize = max_usize_as_size.into();

        // On 64-bit systems, this should be lossless
        // On 32-bit systems, there might be some differences
        if core::mem::size_of::<usize>() == 8 {
            assert_eq!(back_to_usize, usize::MAX);
        }
    }

    #[test]
    fn test_size_ordering() {
        let mut sizes = [
            Size_type::New(300),
            Size_type::New(100),
            Size_type::New(200),
            Size_type::New(50),
        ];

        sizes.sort();

        assert_eq!(sizes[0], Size_type::New(50));
        assert_eq!(sizes[1], Size_type::New(100));
        assert_eq!(sizes[2], Size_type::New(200));
        assert_eq!(sizes[3], Size_type::New(300));
    }
}
