use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    sync::RwLock,
};

use File_system::{Device_trait, Error_type, Path_type, Size_type};

pub struct File_drive_device_type(RwLock<File>);

impl File_drive_device_type {
    pub fn New(Path: &impl AsRef<Path_type>) -> Self {
        let Path = Path.as_ref().As_str();

        let File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
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
            .map_err(|Error| Error.into())
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<File_system::Size_type> {
        self.0
            .try_write()
            .map_err(|_| Error_type::Ressource_busy)?
            .write(Buffer)
            .map(|Size| File_system::Size_type::New(Size as u64))
            .map_err(|Error| Error.into())
    }

    fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
        self.0
            .try_read()
            .map_err(|_| Error_type::Ressource_busy)?
            .metadata()
            .map(|Metadata| Metadata.len())
            .map(Size_type::New)
            .map_err(|Error| Error.into())
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
            .map_err(|Error| Error.into())
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        self.0.write()?.flush().map_err(|Error| Error.into())
    }

    fn Erase(&self) -> File_system::Result_type<()> {
        self.0.write()?.set_len(0).map_err(|Error| Error.into())
    }
}