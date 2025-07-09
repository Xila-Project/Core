use file_system::{Device_trait, Size_type};

pub struct Random_device_type;

impl Random_device_type {
    pub fn new() -> Self {
        Self
    }
}

impl Device_trait for Random_device_type {
    fn Read(&self, buffer: &mut [u8]) -> file_system::Result_type<file_system::Size_type> {
        rand::fill(buffer);

        Ok(buffer.len().into())
    }

    fn Write(&self, _Buffer: &[u8]) -> file_system::Result_type<file_system::Size_type> {
        Err(file_system::Error_type::Unsupported_operation)
    }

    fn get_size(&self) -> file_system::Result_type<file_system::Size_type> {
        Ok(Size_type::New(0))
    }

    fn Set_position(
        &self,
        _position: &file_system::Position_type,
    ) -> file_system::Result_type<file_system::Size_type> {
        Err(file_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> file_system::Result_type<()> {
        Ok(())
    }
}
