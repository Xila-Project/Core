use std::time::SystemTime;

pub struct Time_driver_type {
    Start_time: SystemTime,
}

impl Time_driver_type {
    pub fn New() -> Self {
        Self {
            Start_time: SystemTime::now(),
        }
    }
}

impl Time::Driver_trait for Time_driver_type {
    fn Get_instant_since_startup(&self) -> Time::Duration_type {
        self.Start_time
            .elapsed()
            .expect("Failed to get elapsed time")
    }

    fn Get_current_time(&self) -> Time::Duration_type {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Failed to get current time")
    }
}
