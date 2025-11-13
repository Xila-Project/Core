use std::io::{Read, Write, stderr, stdin, stdout};

use file_system::{BaseOperations, Size};

use crate::io::map_error;

pub struct StandardInDevice;

impl BaseOperations for StandardInDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<Size> {
        #[allow(clippy::unused_io_amount)]
        stdin().read(buffer).unwrap();

        Ok(buffer.len() as _)
    }

    fn write(&self, _: &[u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(0 as _)
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }

    fn is_a_terminal(&self) -> bool {
        true
    }
}

pub struct StandardOutDevice;

impl BaseOperations for StandardOutDevice {
    fn read(&self, _: &mut [u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<Size> {
        Ok(stdout().write(buffer).map_err(map_error)? as _)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(0 as _)
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        stdout().flush().map_err(map_error)
    }

    fn is_a_terminal(&self) -> bool {
        true
    }
}

pub struct StandardErrorDevice;

impl BaseOperations for StandardErrorDevice {
    fn read(&self, _: &mut [u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<Size> {
        Ok(stderr().write(buffer).map_err(map_error)? as _)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(0 as _)
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        stderr().flush().map_err(map_error)
    }

    fn is_a_terminal(&self) -> bool {
        true
    }
}
