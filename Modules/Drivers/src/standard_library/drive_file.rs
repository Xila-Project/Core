use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    sync::RwLock,
};

use file_system::{Device_trait, Error_type, Path_type, Size_type};

use crate::standard_library::io::map_error;

pub struct File_drive_device_type(RwLock<File>);

impl File_drive_device_type {
    pub fn new(path: &impl AsRef<Path_type>) -> Self {
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

impl Device_trait for File_drive_device_type {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result_type<file_system::Size_type> {
        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .read(buffer)
            .map(|size| file_system::Size_type::new(size as u64))
            .map_err(map_error)
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result_type<file_system::Size_type> {
        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .write(buffer)
            .map(|size| file_system::Size_type::new(size as u64))
            .map_err(map_error)
    }

    fn get_size(&self) -> file_system::Result_type<file_system::Size_type> {
        Ok((1024 * 1024 * 1024 * 4_usize).into())
    }

    fn set_position(
        &self,
        position: &file_system::Position_type,
    ) -> file_system::Result_type<file_system::Size_type> {
        let position = match position {
            file_system::Position_type::Start(position) => std::io::SeekFrom::Start(*position),
            file_system::Position_type::End(position) => std::io::SeekFrom::End(*position),
            file_system::Position_type::Current(position) => std::io::SeekFrom::Current(*position),
        };

        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .seek(position)
            .map(Size_type::new)
            .map_err(map_error)
    }

    fn flush(&self) -> file_system::Result_type<()> {
        self.0.write().unwrap().flush().map_err(map_error)
    }

    fn erase(&self) -> file_system::Result_type<()> {
        Ok(())
    }

    fn get_block_size(&self) -> file_system::Result_type<usize> {
        Ok(4096)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use file_system::Device_trait;

    #[test]
    fn test_read_write() {
        let file = File_drive_device_type::new(&"./Test.img");

        let data = [1, 2, 3, 4, 5];

        assert_eq!(file.write(&data).unwrap(), Size_type::new(5));

        file.set_position(&file_system::Position_type::Start(0))
            .unwrap();

        let mut buffer = [0; 5];

        assert_eq!(file.read(&mut buffer).unwrap(), Size_type::new(5));
        assert_eq!(buffer, data);
    }

    #[test]
    fn test_read_write_at_position() {
        let file = File_drive_device_type::new(&"./Test.img");

        file.set_position(&file_system::Position_type::Start(10))
            .unwrap();

        let data = [1, 2, 3, 4, 5];

        assert_eq!(file.write(&data).unwrap(), Size_type::new(5));

        file.set_position(&file_system::Position_type::Start(10))
            .unwrap();

        let mut buffer = [0; 5];

        assert_eq!(file.read(&mut buffer).unwrap(), Size_type::new(5));
        assert_eq!(buffer, data);
    }
}
