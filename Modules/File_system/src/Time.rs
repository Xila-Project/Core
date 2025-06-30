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

#[cfg(test)]
mod Tests {
    use super::*;
    use alloc::{format, vec};

    #[test]
    fn Test_time_creation() {
        let time = Time_type::New(1640995200); // January 1, 2022
        assert_eq!(time.As_u64(), 1640995200);
    }

    #[test]
    fn Test_time_epoch() {
        let epoch = Time_type::New(0);
        assert_eq!(epoch.As_u64(), 0);
    }

    #[test]
    fn Test_time_const_operations() {
        // Test that New and As_u64 are const functions
        const TIME: Time_type = Time_type::New(1234567890);
        const SECONDS: u64 = TIME.As_u64();

        assert_eq!(SECONDS, 1234567890);
        assert_eq!(TIME.As_u64(), 1234567890);
    }

    #[test]
    fn Test_time_comparison() {
        let early = Time_type::New(1000);
        let late = Time_type::New(2000);

        assert!(early < late);
        assert!(late > early);
        assert!(early <= late);
        assert!(late >= early);
        assert!(early <= early);
        assert!(late >= late);
        assert_eq!(early, early);
        assert_ne!(early, late);
    }

    #[test]
    fn Test_time_ordering() {
        let mut times = [
            Time_type::New(3000),
            Time_type::New(1000),
            Time_type::New(2000),
            Time_type::New(500),
        ];

        times.sort();

        assert_eq!(times[0], Time_type::New(500));
        assert_eq!(times[1], Time_type::New(1000));
        assert_eq!(times[2], Time_type::New(2000));
        assert_eq!(times[3], Time_type::New(3000));
    }

    #[test]
    fn Test_time_clone_copy() {
        let original = Time_type::New(999);
        let cloned = original;
        let copied = original;

        assert_eq!(original, cloned);
        assert_eq!(original, copied);
        assert_eq!(cloned, copied);

        // Test that we can still use original after copying
        assert_eq!(original.As_u64(), 999);
    }

    #[test]
    fn Test_time_debug() {
        let time = Time_type::New(1640995200);
        let debug_str = format!("{time:?}");
        assert!(debug_str.contains("Time_type"));
        assert!(debug_str.contains("1640995200"));
    }

    #[test]
    fn Test_time_hash() {
        use alloc::collections::BTreeMap;

        let time1 = Time_type::New(12345);
        let time2 = Time_type::New(12345);
        let time3 = Time_type::New(54321);

        // Test that equal times can be used as keys in collections
        let mut map = BTreeMap::new();
        map.insert(time1, "first");
        map.insert(time2, "second"); // Should overwrite first
        map.insert(time3, "third");

        assert_eq!(map.len(), 2); // time1 and time2 are equal, so only 2 entries
        assert_eq!(map.get(&time1), Some(&"second"));
        assert_eq!(map.get(&time3), Some(&"third"));
    }

    #[test]
    fn Test_time_from_duration() {
        let duration = Duration_type::New(1640995200, 0);
        let time: Time_type = duration.into();
        assert_eq!(time.As_u64(), 1640995200);
    }

    #[test]
    fn Test_time_to_duration() {
        let time = Time_type::New(1640995200);
        let duration: Duration_type = time.into();
        assert_eq!(duration.As_seconds(), 1640995200);
    }

    #[test]
    fn Test_time_display_formatting() {
        // Test display formatting
        let time = Time_type::New(0); // Unix epoch
        let display_str = format!("{time}");

        // The exact format depends on Unix_to_human_time implementation
        // We just verify it produces some reasonable format
        assert!(display_str.contains("-"));
        assert!(display_str.contains(":"));
        assert!(display_str.len() > 10); // Should be a reasonable datetime string
    }

    #[test]
    fn Test_time_display_various_dates() {
        // Test some known timestamps
        let times = vec![
            Time_type::New(0),          // 1970-01-01 00:00:00
            Time_type::New(86400),      // 1970-01-02 00:00:00
            Time_type::New(1640995200), // 2022-01-01 00:00:00 (approximately)
        ];

        for time in times {
            let display_str = format!("{time}");
            // Basic sanity checks
            assert!(display_str.len() >= 19); // YYYY-MM-DD HH:MM:SS is 19 chars
            assert!(display_str.contains("-"));
            assert!(display_str.contains(":"));
            assert!(display_str.contains(" "));
        }
    }

    #[test]
    fn Test_time_max_value() {
        let max_time = Time_type::New(u64::MAX);
        assert_eq!(max_time.As_u64(), u64::MAX);

        // Should still be convertible to duration
        let duration: Duration_type = max_time.into();
        assert_eq!(duration.As_seconds(), u64::MAX);
    }

    #[test]
    fn Test_time_zero_and_max_comparison() {
        let zero = Time_type::New(0);
        let max = Time_type::New(u64::MAX);

        assert!(zero < max);
        assert!(max > zero);
        assert_ne!(zero, max);
    }

    #[test]
    fn Test_time_round_trip_conversions() {
        let original_seconds = 1640995200u64;

        // Time -> Duration -> Time
        let time = Time_type::New(original_seconds);
        let duration: Duration_type = time.into();
        let back_to_time: Time_type = duration.into();

        assert_eq!(time, back_to_time);
        assert_eq!(original_seconds, back_to_time.As_u64());
    }

    #[test]
    fn Test_time_type_size() {
        use core::mem::{align_of, size_of};

        // Should be same size as u64 due to repr(transparent)
        assert_eq!(size_of::<Time_type>(), size_of::<u64>());
        assert_eq!(align_of::<Time_type>(), align_of::<u64>());
    }

    #[test]
    fn Test_time_sequence() {
        // Test a sequence of times
        use alloc::vec::Vec;
        let times: Vec<Time_type> = (0..10)
            .map(|i| Time_type::New(i * 86400)) // Each day
            .collect();

        // Verify they're in ascending order
        for i in 1..times.len() {
            assert!(times[i - 1] < times[i]);
        }
    }
}
