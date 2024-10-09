use core::{
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration,
};

/// Represents a duration of time.
///
/// A duration is the amount of time between two instants. It can only be positive.
/// Its maximum precision is nanoseconds.
/// It is deeply inspired by the [`core::time::Duration`] type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration_type {
    Seconds: u64,
    Nanoseconds: u32,
}

impl Duration_type {
    /// Creates a new [`Duration_type`] from the specified number of seconds and nanoseconds.
    pub const fn New(Seconds: u64, Nanoseconds: u32) -> Self {
        Duration_type {
            Seconds,
            Nanoseconds,
        }
    }

    /// Returns the duration between the two instants.
    ///
    /// # Example
    ///
    /// ```rust
    /// use Shared::Duration_type;
    ///
    /// let earlier = Duration_type::New(1, 0);
    /// let later = Duration_type::New(2, 0);
    ///
    /// let duration = later.Get_duration_since(&earlier);
    ///
    /// assert_eq!(duration, Duration_type::New(1, 0));
    ///
    /// let duration = earlier.Get_duration_since(&later);
    /// assert_eq!(duration, Duration_type::New(0, 0));
    /// ```
    pub fn Get_duration_since(&self, Earlier: &Duration_type) -> Duration_type {
        self.Get_duration_since_checked(Earlier).unwrap_or_default()
    }

    /// Returns the duration between the two instants, or `None` if the duration is negative.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// use Shared::Duration_type;
    ///
    /// let earlier = Duration_type::New(1, 0);
    /// let later = Duration_type::New(2, 0);
    ///
    /// let duration = later.Get_duration_since_checked(&earlier);
    /// assert_eq!(duration, Some(Duration_type::New(1, 0)));
    ///
    /// let duration = earlier.Get_duration_since_checked(&later);
    /// assert_eq!(duration, None);
    /// ```
    pub fn Get_duration_since_checked(&self, Earlier: &Duration_type) -> Option<Duration_type> {
        let self_duration = Duration::new(self.Seconds, self.Nanoseconds);
        let earlier_duration = Duration::new(Earlier.Seconds, Earlier.Nanoseconds);
        self_duration
            .checked_sub(earlier_duration)
            .map(|d| Duration_type {
                Seconds: d.as_secs(),
                Nanoseconds: d.subsec_nanos(),
            })
    }

    /// Returns the duration between the two instants, saturating at the bounds of the type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use Shared::Duration_type;
    ///
    /// let earlier = Duration_type::New(1, 0);
    /// let later = Duration_type::New(2, 0);
    ///
    /// let duration = later.Get_duration_since_saturating(&earlier);
    /// assert_eq!(duration, Duration_type::New(1, 0));
    ///
    /// let duration = earlier.Get_duration_since_saturating(&later);
    /// assert_eq!(duration, Duration_type::default());
    /// ```
    pub fn Get_duration_since_saturating(&self, earlier: &Duration_type) -> Duration_type {
        let self_duration = Duration::new(self.Seconds, self.Nanoseconds);
        let earlier_duration = Duration::new(earlier.Seconds, earlier.Nanoseconds);
        let result = self_duration.saturating_sub(earlier_duration);
        Duration_type {
            Seconds: result.as_secs(),
            Nanoseconds: result.subsec_nanos(),
        }
    }

    pub fn Add_checked(&self, Duration: &Duration_type) -> Option<Duration_type> {
        let self_duration = Duration::new(self.Seconds, self.Nanoseconds);
        let duration = Duration::new(Duration.Seconds, Duration.Nanoseconds);
        self_duration.checked_add(duration).map(|d| Duration_type {
            Seconds: d.as_secs(),
            Nanoseconds: d.subsec_nanos(),
        })
    }

    pub fn Substract_checked(&self, Duration: &Duration_type) -> Option<Duration_type> {
        let self_duration = Duration::new(self.Seconds, self.Nanoseconds);
        let duration = Duration::new(Duration.Seconds, Duration.Nanoseconds);
        self_duration.checked_sub(duration).map(|d| Duration_type {
            Seconds: d.as_secs(),
            Nanoseconds: d.subsec_nanos(),
        })
    }

    pub fn Add_saturating(&self, Duration: &Duration_type) -> Duration_type {
        let self_duration = Duration::new(self.Seconds, self.Nanoseconds);
        let duration = Duration::new(Duration.Seconds, Duration.Nanoseconds);
        let result = self_duration.saturating_add(duration);
        Duration_type {
            Seconds: result.as_secs(),
            Nanoseconds: result.subsec_nanos(),
        }
    }

    pub fn Substract_saturating(&self, duration: &Duration_type) -> Duration_type {
        let self_duration = Duration::new(self.Seconds, self.Nanoseconds);
        let duration = Duration::new(duration.Seconds, duration.Nanoseconds);
        let result = self_duration.saturating_sub(duration);
        Duration_type {
            Seconds: result.as_secs(),
            Nanoseconds: result.subsec_nanos(),
        }
    }

    /// Returns the number of seconds in the duration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use Shared::Duration_type;
    ///
    /// let duration = Duration_type::New(1, 500_000_000);
    /// assert_eq!(duration.As_seconds(), 1);
    /// ```
    pub fn As_seconds(&self) -> u64 {
        self.Seconds
    }

    /// Returns the number of milliseconds in the duration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use Shared::Duration_type;
    ///
    /// let duration = Duration_type::New(1, 500_000_000);
    /// assert_eq!(duration.As_milliseconds(), 1_500);
    /// ```
    pub fn As_milliseconds(&self) -> u64 {
        self.As_microseconds() as u64 / 1_000
    }

    /// Returns the number of microseconds in the duration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use Shared::Duration_type;
    ///
    /// let duration = Duration_type::New(1, 500_000_000);
    ///
    /// assert_eq!(duration.As_microseconds(), 1_500_000);
    /// ```
    pub fn As_microseconds(&self) -> u128 {
        self.As_nanoseconds() / 1_000
    }

    /// Returns the number of nanoseconds in the duration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use Shared::Duration_type;
    ///
    /// let duration = Duration_type::New(1, 500_000_000);
    /// assert_eq!(duration.As_nanoseconds(), 1_500_000_000);
    /// ```
    pub fn As_nanoseconds(&self) -> u128 {
        u128::from(self.Seconds) * 1_000_000_000 + u128::from(self.Nanoseconds)
    }
}

impl Add<&Duration_type> for Duration_type {
    type Output = Duration_type;

    fn add(self, Duration: &Duration_type) -> Duration_type {
        self.Add_checked(Duration)
            .expect("Overflow when adding duration")
    }
}

impl AddAssign<&Duration_type> for Duration_type {
    fn add_assign(&mut self, Duration: &Duration_type) {
        *self = self
            .Add_checked(Duration)
            .expect("Overflow when adding duration");
    }
}

impl Sub<&Duration_type> for Duration_type {
    type Output = Duration_type;

    fn sub(self, Duration: &Duration_type) -> Duration_type {
        self.Substract_checked(Duration)
            .expect("Overflow when substracting duration")
    }
}

impl SubAssign<&Duration_type> for Duration_type {
    fn sub_assign(&mut self, Duration: &Duration_type) {
        *self = self
            .Substract_checked(Duration)
            .expect("Overflow when substracting duration");
    }
}

impl AsMut<[u8]> for Duration_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, core::mem::size_of::<Self>())
        }
    }
}

impl AsRef<[u8]> for Duration_type {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of::<Self>())
        }
    }
}
