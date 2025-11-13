use file_system::{BaseOperations, Size};

pub struct NullDevice;

impl BaseOperations for NullDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        Ok(buffer.len() as _)
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<file_system::Size> {
        Ok(buffer.len() as _)
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        Ok(0 as _)
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<file_system::Size> {
        Ok(0 as _)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
