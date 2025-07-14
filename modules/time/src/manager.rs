use std::sync::OnceLock;

use core::time::Duration;
use file_system::Device;

use crate::{Error, Result};

pub static MANAGER: OnceLock<Manager> = OnceLock::new();

pub fn get_instance() -> &'static Manager {
    MANAGER.get().expect("Time manager is not initialized")
}

pub fn initialize(driver: Device) -> Result<&'static Manager> {
    MANAGER.get_or_init(|| Manager::new(driver).expect("Failed to initialize time manager"));

    Ok(get_instance())
}

pub struct Manager {
    device: Device,
    start_time: Duration,
}

impl Manager {
    pub fn new(device: Device) -> Result<Self> {
        let start_time = Self::get_current_time_from_device(&device)?;

        Ok(Self { device, start_time })
    }

    pub fn get_current_time_since_startup(&self) -> Result<Duration> {
        let current_time = self.get_current_time()?;

        Ok(current_time.abs_diff(self.start_time))
    }

    pub fn get_current_time(&self) -> Result<Duration> {
        Self::get_current_time_from_device(&self.device)
    }

    fn get_current_time_from_device(device: &Device) -> Result<Duration> {
        let mut current_time = Duration::default();

        let current_time_raw = unsafe {
            core::slice::from_raw_parts_mut(
                &mut current_time as *mut Duration as *mut u8,
                core::mem::size_of::<Duration>(),
            )
        };

        device.read(current_time_raw).map_err(Error::DeviceError)?;

        Ok(current_time)
    }
}
