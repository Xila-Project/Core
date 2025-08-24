use std::time::{SystemTime, UNIX_EPOCH};

use core::time::Duration;
use file_system::{DeviceTrait, Error, Result, Size};

pub struct TimeDevice;

impl Default for TimeDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeDevice {
    pub fn new() -> Self {
        Self {}
    }
}

impl DeviceTrait for TimeDevice {
    fn read(&self, buffer: &mut [u8]) -> Result<Size> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::InternalError)?;

        let duration: Duration = Duration::new(duration.as_secs(), duration.subsec_nanos());

        let duration_bytes = unsafe {
            core::slice::from_raw_parts(
                &duration as *const Duration as *const u8,
                core::mem::size_of::<Duration>(),
            )
        };

        buffer.copy_from_slice(duration_bytes);

        Ok(buffer.len().into())
    }

    fn write(&self, _: &[u8]) -> Result<file_system::Size> {
        Err(Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        Ok(size_of::<Duration>().into())
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<file_system::Size> {
        Err(Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Err(Error::UnsupportedOperation)
    }
}
