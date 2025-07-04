use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    sync::RwLock,
};

use File_system::{Device_trait, Error_type, Path_type, Size_type};

use crate::Std::IO::Map_error;

pub struct File_drive_device_type(RwLock<File>);

impl File_drive_device_type {
    pub fn New(Path: &impl AsRef<Path_type>) -> Self {
        let Path = Path.as_ref().As_str();

        let File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(Path)
            .expect("Error opening file");

        Self(RwLock::new(File))
    }
}

impl Device_trait for File_drive_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .read(Buffer)
            .map(|Size| File_system::Size_type::New(Size as u64))
            .map_err(Map_error)
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .write(Buffer)
            .map(|Size| File_system::Size_type::New(Size as u64))
            .map_err(Map_error)
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        Ok((1024 * 1024 * 1024 * 4_usize).into())
    }

    fn Set_position(
        &self,
        Position: &File_system::Position_type,
    ) -> File_system::Result_type<File_system::Size_type> {
        let Position = match Position {
            File_system::Position_type::Start(Position) => std::io::SeekFrom::Start(*Position),
            File_system::Position_type::End(Position) => std::io::SeekFrom::End(*Position),
            File_system::Position_type::Current(Position) => std::io::SeekFrom::Current(*Position),
        };

        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .seek(Position)
            .map(Size_type::New)
            .map_err(Map_error)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        self.0.write().unwrap().flush().map_err(Map_error)
    }

    fn Erase(&self) -> File_system::Result_type<()> {
        Ok(())
    }

    fn Get_block_size(&self) -> File_system::Result_type<usize> {
        Ok(4096)
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use File_system::Device_trait;

    #[test]
    fn Test_read_write() {
        let File = File_drive_device_type::New(&"./Test.img");

        let Data = [1, 2, 3, 4, 5];

        assert_eq!(File.Write(&Data).unwrap(), Size_type::New(5));

        File.Set_position(&File_system::Position_type::Start(0))
            .unwrap();

        let mut Buffer = [0; 5];

        assert_eq!(File.Read(&mut Buffer).unwrap(), Size_type::New(5));
        assert_eq!(Buffer, Data);
    }

    #[test]
    fn Test_read_write_at_position() {
        let File = File_drive_device_type::New(&"./Test.img");

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
