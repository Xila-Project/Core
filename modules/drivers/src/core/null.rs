use file_system::{DeviceTrait, Size};

pub struct NullDevice;

impl DeviceTrait for NullDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        Ok(Size::new(buffer.len() as u64))
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<file_system::Size> {
        Ok(Size::new(buffer.len() as u64))
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        Ok(Size::new(0))
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<file_system::Size> {
        Ok(Size::new(0))
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
