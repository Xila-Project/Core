use core::fmt::{self, Display, Formatter};

use Shared::{Duration_type, Unix_to_human_time};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Time_type {
    Seconds: u64,
}

impl Time_type {
    pub const fn New(Seconds: u64) -> Self {
        Self { Seconds }
    }
}

impl From<Duration_type> for Time_type {
    fn from(Duration: Duration_type) -> Self {
        Self {
            Seconds: Duration.As_seconds(),
        }
    }
}

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
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            Year, Month, Day, Hour, Minute, Second
        )
    }
}
