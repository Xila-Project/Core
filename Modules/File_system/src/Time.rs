use std::time::Duration;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time_type {
    Seconds: u64,
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

pub fn Get_now() -> Time_type {
    Time::Get_instance().Get_current_time().into()
}
