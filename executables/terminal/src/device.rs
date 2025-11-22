use alloc::string::String;

use xila::file_system::{
    DirectBaseOperations, DirectCharacterDevice, Error, MountOperations, Position, Result, Size,
};

use crate::terminal::Terminal;

fn map_error(error: crate::Error) -> Error {
    match error {
        crate::Error::RessourceBusy => Error::RessourceBusy,
        _ => Error::InternalError,
    }
}

impl DirectBaseOperations for Terminal {
    fn read(&self, buffer: &mut [u8], _: Size) -> Result<usize> {
        self.read_input(buffer).map_err(map_error)
    }

    fn write(&self, buffer: &[u8], _: Size) -> Result<usize> {
        let string = String::from_utf8_lossy(buffer);

        self.print(&string).map_err(map_error)?;

        Ok(buffer.len())
    }

    fn set_position(&self, _: Size, _: &Position) -> Result<Size> {
        Err(Error::UnsupportedOperation)
    }
}

impl MountOperations for Terminal {}

impl DirectCharacterDevice for Terminal {}
