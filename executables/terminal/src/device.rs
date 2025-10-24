use alloc::string::String;

use xila::file_system::{self, DeviceTrait};
use xila::futures::block_on;

use crate::terminal::Terminal;

impl DeviceTrait for Terminal {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        block_on(self.read_input(buffer)).map_err(|_| file_system::Error::InternalError)
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<file_system::Size> {
        let string = String::from_utf8_lossy(buffer);

        block_on(self.print(&string)).map_err(|_| file_system::Error::InternalError)?;

        Ok(buffer.len().into())
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        Ok(0_usize.into())
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<file_system::Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
