use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
};

use file_system::{
    ControlArgument, ControlCommand, DirectBaseOperations, DirectBlockDevice, Error,
    MountOperations, Path, Position, Size, block_device, mount::MutexMountWrapper,
};
use synchronization::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::io::map_error;

pub struct FileDriveDevice(MutexMountWrapper<CriticalSectionRawMutex, File>);

impl FileDriveDevice {
    pub fn new(path: &impl AsRef<Path>) -> Self {
        let path = path.as_ref().as_str();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .expect("Error opening file");

        file.set_len(16 * 1024 * 1024)
            .expect("Error setting file size");

        Self(MutexMountWrapper::new_mounted(file))
    }

    pub fn new_static(path: &impl AsRef<Path>) -> &'static Self {
        Box::leak(Box::new(Self::new(path)))
    }
}

impl DirectBaseOperations for FileDriveDevice {
    fn read(&self, buffer: &mut [u8], position: Size) -> file_system::Result<usize> {
        let mut inner = self.0.try_get()?;

        inner.seek(SeekFrom::Start(position)).map_err(map_error)?;
        inner.read(buffer).map_err(map_error)
    }

    fn write(&self, buffer: &[u8], position: Size) -> file_system::Result<usize> {
        let mut inner = self.0.try_get()?;

        inner.seek(SeekFrom::Start(position)).map_err(map_error)?;
        inner.write(buffer).map_err(map_error)
    }

    fn write_pattern(
        &self,
        pattern: &[u8],
        count: usize,
        absolute_position: Size,
    ) -> file_system::Result<usize> {
        let mut inner = self.0.try_get()?;

        inner
            .seek(SeekFrom::Start(absolute_position))
            .map_err(map_error)?;

        for _ in 0..count {
            inner.write_all(pattern).map_err(map_error)?;
        }

        Ok(pattern.len() * count)
    }

    fn set_position(&self, _: Size, position: &Position) -> file_system::Result<Size> {
        let position = match position {
            Position::Start(position) => SeekFrom::Start(*position),
            Position::End(position) => SeekFrom::End(*position),
            Position::Current(position) => SeekFrom::Current(*position),
        };

        self.0
            .try_get()
            .map_err(|_| Error::RessourceBusy)?
            .seek(position)
            .map_err(map_error)
    }

    fn flush(&self) -> file_system::Result<()> {
        self.0
            .try_get()
            .map_err(|_| Error::RessourceBusy)?
            .flush()
            .map_err(map_error)
    }

    fn control(
        &self,
        command: ControlCommand,
        argument: &mut ControlArgument,
    ) -> file_system::Result<()> {
        match command {
            block_device::GET_BLOCK_SIZE => {
                let block_size = argument.cast::<usize>().ok_or(Error::InvalidParameter)?;

                *block_size = 512; // Fixed block size for file drive device

                Ok(())
            }
            block_device::GET_BLOCK_COUNT => {
                let block_count = argument.cast::<Size>().ok_or(Error::InvalidParameter)?;

                let file_size = self
                    .0
                    .try_get()
                    .map_err(|_| Error::RessourceBusy)?
                    .metadata()
                    .map_err(map_error)?
                    .len();

                *block_count = file_size / 512; // Fixed block size for file drive device

                Ok(())
            }
            _ => Err(Error::UnsupportedOperation),
        }
    }
}

impl MountOperations for FileDriveDevice {
    fn unmount(&self) -> file_system::Result<()> {
        self.0.unmount()
    }
}

impl DirectBlockDevice for FileDriveDevice {}

#[cfg(test)]
mod tests {
    use super::*;
    use file_system::implement_block_device_tests;

    implement_block_device_tests!(FileDriveDevice::new(&Path::from_str(
        "/tmp/file_drive_device_test.img"
    )));
}
