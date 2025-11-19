use file_system::{CharacterDevice, DirectBaseOperations, MountOperations, Size};

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

impl DirectBaseOperations for RandomDevice {
    fn read(&self, buffer: &mut [u8], _: Size) -> file_system::Result<usize> {
        getrandom::fill(buffer).map_err(|_| file_system::Error::Other)?;

        Ok(buffer.len())
    }

    fn write(&self, _: &[u8], _: Size) -> file_system::Result<usize> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn set_position(
        &self,
        _: Size,
        _position: &file_system::Position,
    ) -> file_system::Result<file_system::Size> {
        Err(file_system::Error::UnsupportedOperation)
    }
}

impl MountOperations for RandomDevice {}

impl CharacterDevice for RandomDevice {}
