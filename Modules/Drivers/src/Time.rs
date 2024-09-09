use File_system::Device_trait;

use std::time::SystemTime;

pub struct Time_device_type;

impl Device_trait for Time_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<usize> {
        let System_time = SystemTime::now();
        let Duration = System_time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let Time = Duration.as_secs();
        let Time = Time.to_be_bytes();
        Buffer.copy_from_slice(&Time);
        Ok(8)
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<usize> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<usize> {
        Ok(8)
    }

    fn Set_position(
        &self,
        Position: &File_system::Position_type,
    ) -> File_system::Result_type<usize> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}
