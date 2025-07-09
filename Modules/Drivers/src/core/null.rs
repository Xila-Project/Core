use file_system::{Device_trait, Size_type};

pub struct Null_device_type;

impl Device_trait for Null_device_type {
    fn Read(&self, buffer: &mut [u8]) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::New(buffer.len() as u64))
    }

    fn Write(&self, Buffer: &[u8]) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::New(Buffer.len() as u64))
    }

    fn get_size(&self) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(
        &self,
        _: &file_system::Position_type,
    ) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::New(0))
    }

    fn Flush(&self) -> file_system::Result_type<()> {
        Ok(())
    }
}
