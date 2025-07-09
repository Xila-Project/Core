use file_system::{Device_trait, Size_type};

pub struct Null_device_type;

impl Device_trait for Null_device_type {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::new(buffer.len() as u64))
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::new(buffer.len() as u64))
    }

    fn get_size(&self) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::new(0))
    }

    fn set_position(
        &self,
        _: &file_system::Position_type,
    ) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::new(0))
    }

    fn flush(&self) -> file_system::Result_type<()> {
        Ok(())
    }
}
