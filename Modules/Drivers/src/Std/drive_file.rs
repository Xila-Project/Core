use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    sync::RwLock,
};

use file_system::{Device_trait, Error_type, Path_type, Size_type};

use crate::Std::IO::Map_error;

pub struct File_drive_device_type(RwLock<File>);

impl File_drive_device_type {
    pub fn new(path: &impl AsRef<Path_type>) -> Self {
        let path = path.as_ref().As_str();

        let File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .expect("Error opening file");

        Self(RwLock::new(File))
    }
}

impl Device_trait for File_drive_device_type {
    fn Read(&self, buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .read(buffer)
            .map(|size| File_system::Size_type::New(size as u64))
            .map_err(Map_error)
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .write(Buffer)
            .map(|size| File_system::Size_type::New(size as u64))
            .map_err(Map_error)
    }

    fn get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok((1024 * 1024 * 1024 * 4_usize).into())
    }

    fn Set_position(
        &self,
        position: &File_system::Position_type,
    ) -> File_system::Result_type<File_system::Size_type> {
        let position = match position {
            File_system::Position_type::Start(Position) => std::io::SeekFrom::Start(*Position),
            File_system::Position_type::End(position) => std::io::SeekFrom::End(*position),
            File_system::Position_type::Current(position) => std::io::SeekFrom::Current(*position),
        };

        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .seek(position)
            .map(Size_type::New)
            .map_err(Map_error)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        self.0.write().unwrap().flush().map_err(Map_error)
    }

    fn Erase(&self) -> File_system::Result_type<()> {
        Ok(())
    }

    fn get_block_size(&self) -> File_system::Result_type<usize> {
        Ok(4096)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use file_system::Device_trait;

    #[test]
    fn test_read_write() {
        let File = File_drive_device_type::new(&"./Test.img");

        let Data = [1, 2, 3, 4, 5];

        assert_eq!(File.Write(&Data).unwrap(), Size_type::New(5));

        File.Set_position(&File_system::Position_type::Start(0))
            .unwrap();

        let mut Buffer = [0; 5];

        assert_eq!(File.Read(&mut Buffer).unwrap(), Size_type::New(5));
        assert_eq!(Buffer, Data);
    }

    #[test]
    fn test_read_write_at_position() {
        let File = File_drive_device_type::new(&"./Test.img");

        File.Set_position(&File_system::Position_type::Start(10))
            .unwrap();

        let Data = [1, 2, 3, 4, 5];

        assert_eq!(File.Write(&Data).unwrap(), Size_type::New(5));

        File.Set_position(&File_system::Position_type::Start(10))
            .unwrap();

        let mut Buffer = [0; 5];

        assert_eq!(File.Read(&mut Buffer).unwrap(), Size_type::New(5));
        assert_eq!(Buffer, Data);
    }
}
