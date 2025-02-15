use std::sync::OnceLock;

use File_system::Device_type;
use Shared::Duration_type;

use crate::{Error_type, Result_type};

pub struct Manager_type {
    Device: Device_type,
    Start_time: Duration_type,
}

impl Manager_type {
    pub fn New(Device: Device_type) -> Result_type<Self> {
        let mut Start_time = Duration_type::default();

        Device
            .Read(Start_time.as_mut())
            .map_err(Error_type::Device_error)?;

        Ok(Self { Device, Start_time })
    }

    pub fn Get_current_time_since_startup(&self) -> Result_type<Duration_type> {
        let Current_time = self.Get_current_time()?;

        Ok(Current_time.Get_duration_since(&self.Start_time))
    }

    pub fn Get_current_time(&self) -> Result_type<Duration_type> {
        let mut Current_time = Duration_type::default();

        self.Device
            .Read(Current_time.as_mut())
            .map_err(Error_type::Device_error)?;

        Ok(Current_time)
    }
}
