use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Time_type {
    Seconds: u64,
}

impl Time_type {
    pub const fn New(Seconds: u64) -> Self {
        Self { Seconds }
    }

    pub fn Get_now() -> Self {
        Time::Get_instance().Get_current_time().into()
    }
}

impl From<Duration> for Time_type {
    fn from(Duration: Duration) -> Self {
        Self {
            Seconds: Duration.as_secs(),
        }
    }
}

impl From<Time_type> for Duration {
    fn from(Time: Time_type) -> Self {
        Self::from_secs(Time.Seconds)
    }
}
