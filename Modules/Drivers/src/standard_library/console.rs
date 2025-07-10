use std::io::{stderr, stdin, stdout, Read, Write};

use file_system::{DeviceTrait, Size};

use crate::standard_library::io::map_error;

pub struct StandardInDevice;

impl DeviceTrait for StandardInDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<Size> {
        #[allow(clippy::unused_io_amount)]
        stdin().read(buffer).unwrap();

        Ok(Size::new(buffer.len() as u64))
    }

    fn write(&self, _: &[u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(Size::new(0))
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

pub struct StandardOutDeviceType;

impl DeviceTrait for StandardOutDeviceType {
    fn read(&self, _: &mut [u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<Size> {
        Ok(Size::new(stdout().write(buffer).map_err(map_error)? as u64))
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(Size::new(0))
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

pub struct StandardErrorDeviceType;

impl DeviceTrait for StandardErrorDeviceType {
    fn read(&self, _: &mut [u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<Size> {
        Ok(Size::new(stderr().write(buffer).map_err(map_error)? as u64))
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(Size::new(0))
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
