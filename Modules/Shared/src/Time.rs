use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Time_type {
    Seconds: u64,
}

impl Time_type {
    pub fn New(Seconds: u64) -> Self {
        Time_type { Seconds }
    }

    pub fn Get_now() -> Self {
        let System_time = SystemTime::now();
        let Duration = System_time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        Time_type::New(Duration.as_secs())
    }

    pub const fn Get_seconds(&self) -> u64 {
        self.Seconds
    }

    pub fn Set_seconds(&mut self, Seconds: u64) -> &mut Self {
        self.Seconds = Seconds;
        self
    }
}
