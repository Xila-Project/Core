use std::sync::OnceLock;

use file_system::Device_type;
use shared::Duration_type;

use crate::{Error_type, Result_type};

pub static MANAGER: OnceLock<Manager_type> = OnceLock::new();

pub fn get_instance() -> &'static Manager_type {
    MANAGER.get().expect("Time manager is not initialized")
}

pub fn initialize(Driver: Device_type) -> Result_type<&'static Manager_type> {
    MANAGER.get_or_init(|| Manager_type::new(Driver).expect("Failed to initialize time manager"));

    Ok(get_instance())
}

pub struct Manager_type {
    device: Device_type,
    start_time: Duration_type,
}

impl Manager_type {
    pub fn new(device: Device_type) -> Result_type<Self> {
        let mut start_time = Duration_type::default();

        device
            .Read(start_time.as_mut())
            .map_err(Error_type::Device_error)?;

        Ok(Self { device, start_time })
    }

    pub fn get_current_time_since_startup(&self) -> Result_type<Duration_type> {
        let current_time = self.get_current_time()?;

        Ok(current_time.get_duration_since(&self.start_time))
    }

    pub fn get_current_time(&self) -> Result_type<Duration_type> {
        let mut current_time = Duration_type::default();

        self.device
            .Read(current_time.as_mut())
            .map_err(Error_type::Device_error)?;

        Ok(current_time)
    }
}
