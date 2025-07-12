use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    sync::RwLock,
};

use file_system::{DeviceTrait, Error, Path, Size};

use crate::standard_library::io::map_error;

pub struct FileDriveDeviceType(RwLock<File>);

impl FileDriveDeviceType {
    pub fn new(path: &impl AsRef<Path>) -> Self {
        let path = path.as_ref().as_str();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .expect("Error opening file");

        Self(RwLock::new(file))
    }
}

impl DeviceTrait for FileDriveDeviceType {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        self.0
            .try_write()
            .map_err(|_| Error::RessourceBusy)?
            .read(buffer)
            .map(|size| file_system::Size::new(size as u64))
            .map_err(map_error)
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<file_system::Size> {
        self.0
            .try_write()
            .map_err(|_| Error::RessourceBusy)?
            .write(buffer)
            .map(|size| file_system::Size::new(size as u64))
            .map_err(map_error)
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        Ok((1024 * 1024 * 1024 * 4_usize).into())
    }

    fn set_position(
        &self,
        position: &file_system::Position,
    ) -> file_system::Result<file_system::Size> {
        let position = match position {
            file_system::Position::Start(position) => std::io::SeekFrom::Start(*position),
            file_system::Position::End(position) => std::io::SeekFrom::End(*position),
            file_system::Position::Current(position) => std::io::SeekFrom::Current(*position),
        };

        self.0
            .try_write()
            .map_err(|_| Error::RessourceBusy)?
            .seek(position)
            .map(Size::new)
            .map_err(map_error)
    }

    fn flush(&self) -> file_system::Result<()> {
        self.0.write().unwrap().flush().map_err(map_error)
    }

    fn erase(&self) -> file_system::Result<()> {
        Ok(())
    }

    fn get_block_size(&self) -> file_system::Result<usize> {
        Ok(4096)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use file_system::DeviceTrait;

    #[test]
    fn test_read_write() {
        let file = FileDriveDeviceType::new(&"./Test.img");

        let data = [1, 2, 3, 4, 5];

        assert_eq!(file.write(&data).unwrap(), Size::new(5));

        file.set_position(&file_system::Position::Start(0)).unwrap();

        let mut buffer = [0; 5];

        assert_eq!(file.read(&mut buffer).unwrap(), Size::new(5));
        assert_eq!(buffer, data);
    }

    #[test]
    fn test_read_write_at_position() {
        let file = FileDriveDeviceType::new(&"./Test.img");

        file.set_position(&file_system::Position::Start(10))
            .unwrap();

        let data = [1, 2, 3, 4, 5];

        assert_eq!(file.write(&data).unwrap(), Size::new(5));

        file.set_position(&file_system::Position::Start(10))
            .unwrap();

        let mut buffer = [0; 5];

        assert_eq!(file.read(&mut buffer).unwrap(), Size::new(5));
        assert_eq!(buffer, data);
    }
}
