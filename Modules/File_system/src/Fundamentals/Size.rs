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
/// use File_system::Size_type;
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
    /// use File_system::Size_type;
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
    /// use File_system::Size_type;
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
