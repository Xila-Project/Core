use file_system::{CharacterDevice, DirectBaseOperations, MountOperations, Size};

pub struct NullDevice;

impl DirectBaseOperations for NullDevice {
    fn read(&self, buffer: &mut [u8], _: Size) -> file_system::Result<usize> {
        Ok(buffer.len() as _)
    }

    fn write(&self, buffer: &[u8], _: Size) -> file_system::Result<usize> {
        Ok(buffer.len() as _)
    }
}

impl MountOperations for NullDevice {}

impl CharacterDevice for NullDevice {}
