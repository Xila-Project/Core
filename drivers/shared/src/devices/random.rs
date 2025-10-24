use file_system::{DeviceTrait, Size};

pub struct RandomDevice;

impl Default for RandomDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomDevice {
    pub fn new() -> Self {
        Self
    }
}

impl DeviceTrait for RandomDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        getrandom::fill(buffer).map_err(|_| file_system::Error::Other)?;

        Ok(buffer.len().into())
    }

    fn write(&self, _buffer: &[u8]) -> file_system::Result<file_system::Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        Ok(Size::new(0))
    }

    fn set_position(
        &self,
        _position: &file_system::Position,
    ) -> file_system::Result<file_system::Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
