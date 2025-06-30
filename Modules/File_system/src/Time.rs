//! Time representation and utilities for file system operations.
//!
//! This module provides time-related types and functionality specifically designed
//! for file system metadata operations, including creation times, modification times,
//! and access times.

use core::fmt::{self, Display, Formatter};

use Shared::{Duration_type, Unix_to_human_time};

/// Represents a point in time for file system operations.
///
/// `Time_type` stores time as seconds since the Unix epoch (January 1, 1970).
/// It's used throughout the file system for tracking file creation, modification,
/// and access times. The type is designed to be efficient for storage and comparison.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// use File_system::Time_type;
///
/// // Create a time representing the Unix epoch
/// let epoch = Time_type::New(0);
/// assert_eq!(epoch.As_u64(), 0);
///
/// // Create a time for a specific moment
/// let time = Time_type::New(1642684800); // January 20, 2022
/// ```
///
/// # Storage
///
/// Times are stored as 64-bit unsigned integers representing seconds, providing
/// a range from 1970 to approximately year 584 billion, which is sufficient
/// for any practical file system use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Time_type {
    Seconds: u64,
}

impl Time_type {
    /// Create a new time from seconds since Unix epoch.
    ///
    /// # Arguments
    ///
    /// * `Seconds` - Number of seconds since January 1, 1970 00:00:00 UTC
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// use File_system::Time_type;
    ///
    /// let time = Time_type::New(1640995200); // January 1, 2022
    /// ```
    pub const fn New(Seconds: u64) -> Self {
        Self { Seconds }
    }

    /// Get the time as seconds since Unix epoch.
    ///
    /// # Returns
    ///
    /// Number of seconds since January 1, 1970 00:00:00 UTC.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// use File_system::Time_type;
    ///
    /// let time = Time_type::New(1640995200);
    /// assert_eq!(time.As_u64(), 1640995200);
    /// ```
    pub const fn As_u64(self) -> u64 {
        self.Seconds
    }
}

/// Convert from a duration to a time.
///
/// This treats the duration as an absolute time since the Unix epoch.
impl From<Duration_type> for Time_type {
    fn from(Duration: Duration_type) -> Self {
        Self {
            Seconds: Duration.As_seconds(),
        }
    }
}

/// Convert from a time to a duration.
///
/// This converts the absolute time to a duration since the Unix epoch.
impl From<Time_type> for Duration_type {
    fn from(Time: Time_type) -> Self {
        Duration_type::New(Time.Seconds, 0)
    }
}

impl Display for Time_type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (Year, Month, Day, Hour, Minute, Second) = Unix_to_human_time(self.Seconds as i64);

        write!(
            f,
            "{Year:04}-{Month:02}-{Day:02} {Hour:02}:{Minute:02}:{Second:02}",
        )
    }
}
