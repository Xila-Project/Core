use std::time::{SystemTime, UNIX_EPOCH};

use File_system::{Device_trait, Error_type, Result_type, Size_type};
use Shared::Duration_type;

pub struct Time_driver_type;

impl Time_driver_type {
    pub fn new() -> Self {
        Self {}
    }
}

impl Device_trait for Time_driver_type {
    fn Read(&self, buffer: &mut [u8]) -> Result_type<Size_type> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error_type::Internal_error)?;

        let Duration = Duration_type::New(duration.as_secs(), duration.subsec_nanos());

        buffer.copy_from_slice(Duration.as_ref());

        Ok(buffer.len().into())
    }

    fn Write(&self, _: &[u8]) -> Result_type<File_system::Size_type> {
        Err(Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok(size_of::<Duration_type>().into())
    }

    fn Set_position(
        &self,
        _: &File_system::Position_type,
    ) -> File_system::Result_type<File_system::Size_type> {
        Err(Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }
}
