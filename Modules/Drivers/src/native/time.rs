use std::time::{SystemTime, UNIX_EPOCH};

use file_system::{Device_trait, Error_type, Result_type, Size_type};
use shared::Duration_type;

pub struct Time_driver_type;

impl Time_driver_type {
    pub fn new() -> Self {
        Self {}
    }
}

impl Device_trait for Time_driver_type {
    fn read(&self, buffer: &mut [u8]) -> Result_type<Size_type> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error_type::Internal_error)?;

        let duration: Duration_type =
            Duration_type::new(duration.as_secs(), duration.subsec_nanos());

        buffer.copy_from_slice(duration.as_ref());

        Ok(buffer.len().into())
    }

    fn write(&self, _: &[u8]) -> Result_type<file_system::Size_type> {
        Err(Error_type::Unsupported_operation)
    }

    fn get_size(&self) -> file_system::Result_type<file_system::Size_type> {
        Ok(size_of::<Duration_type>().into())
    }

    fn set_position(
        &self,
        _: &file_system::Position_type,
    ) -> file_system::Result_type<file_system::Size_type> {
        Err(Error_type::Unsupported_operation)
    }

    fn flush(&self) -> file_system::Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }
}
