use core::time::Duration;
use file_system::{
    DirectBaseOperations, DirectCharacterDevice, Error, MountOperations, Result, Size,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TimeDevice;

impl DirectBaseOperations for TimeDevice {
    fn read(&self, buffer: &mut [u8], _: Size) -> Result<usize> {
        if buffer.len() < size_of::<Duration>() {
            return Err(Error::InvalidParameter);
        }

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::InternalError)?;

        let duration: Duration = Duration::new(duration.as_secs(), duration.subsec_nanos());

        let duration_bytes = unsafe {
            core::slice::from_raw_parts(
                &duration as *const Duration as *const u8,
                size_of::<Duration>(),
            )
        };

        buffer[..duration_bytes.len()].copy_from_slice(duration_bytes);

        Ok(buffer.len())
    }

    fn write(&self, _: &[u8], _: Size) -> Result<usize> {
        Err(Error::UnsupportedOperation)
    }
}

impl MountOperations for TimeDevice {}

impl DirectCharacterDevice for TimeDevice {}
