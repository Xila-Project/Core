use super::*;

pub enum Mode_type {
    Read,
    Write,
    Read_write,
    Append,
    Read_append,
}

pub enum Type_type {
    File,
    Directory,
    Symbolic_link,
}

pub type File_identifier_type = u16;

pub struct File_type<'a> {
    File_identifier: File_identifier_type,
    File_system: &'a dyn File_system_traits,
}

impl<'a> File_type<'a> {
    pub fn New(
        File_identifier: File_identifier_type,
        File_system: &'a dyn File_system_traits,
    ) -> Self {
        Self {
            File_identifier,
            File_system,
        }
    }
}

impl<'a> File_type<'a> {
    // - Operations
    pub fn Get_identifier(&self) -> File_identifier_type {
        self.File_identifier
    }

    pub fn Set_position(&mut self, Offset: Size_type) -> Result<Size_type, Error_type> {
        self.File_system
            .Set_file_position(self.Get_identifier(), Position_type::Start(Offset))
    }
    pub fn Write(&self, Buffer: &[u8]) -> Result<usize, Error_type> {
        self.File_system.Write_file(self.Get_identifier(), Buffer)
    }
    pub fn Write_line(&self, Buffer: &[u8]) -> Result<usize, Error_type> {
        let Size = self.File_system.Write_file(self.Get_identifier(), Buffer)?;
        Ok(Size + self.File_system.Write_file(self.Get_identifier(), b"\n")?)
    }
    pub fn Read(&self, Buffer: &mut [u8]) -> Result<usize, Error_type> {
        self.File_system.Read_file(self.Get_identifier(), Buffer)
    }
    pub fn Read_line(&self, Buffer: &mut [u8]) -> Result<(), Error_type> {
        let mut Buffer = Buffer.iter_mut();
        loop {
            let Byte = self
                .File_system
                .Read_file(self.Get_identifier(), &mut [0; 1])?;

            if Byte == 0 {
                return Ok(());
            }
            let Byte = Buffer.next().unwrap();
            if *Byte == b'\n' {
                break;
            }
        }
        Ok(())
    }

    pub fn Read_vector(&self) -> Result<Vec<u8>, Error_type> {
        let Size = self.Get_size()?;
        let mut Buffer = vec![0; Size.0 as usize];
        self.Read(&mut Buffer).map(|_| Buffer)
    }

    pub fn Get_position(&self) -> Size_type {
        self.File_system
            .Get_file_position(self.Get_identifier())
            .unwrap()
    }

    // - Metadata
    pub fn Get_size(&self) -> Result<Size_type, Error_type> {
        self.File_system.Get_file_size(self.Get_identifier())
    }

    pub fn Get_type(&self) -> Result<Type_type, Error_type> {
        self.File_system.Get_file_type(self.Get_identifier())
    }
}

impl std::io::Read for File_type<'_> {
    fn read(&mut self, Buffer: &mut [u8]) -> Result<usize, std::io::Error> {
        self.File_system
            .Read_file(self.File_identifier, Buffer)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Error"))
    }
}

impl std::io::Write for File_type<'_> {
    fn write(&mut self, Buffer: &[u8]) -> Result<usize, std::io::Error> {
        self.File_system
            .Write_file(self.File_identifier, Buffer)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Error"))
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.File_system
            .Flush_file(self.File_identifier)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Error"))
    }
}

impl std::io::Seek for File_type<'_> {
    fn seek(&mut self, Position: std::io::SeekFrom) -> Result<u64, std::io::Error> {
        match Position {
            std::io::SeekFrom::Start(Offset) => self
                .File_system
                .Set_file_position(self.File_identifier, Position_type::Start(Offset.into()))
                .map(|x| x.0)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Error")),
            std::io::SeekFrom::End(Offset) => self
                .File_system
                .Set_file_position(self.File_identifier, Position_type::End(Offset))
                .map(|x| x.0)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Error")),
            std::io::SeekFrom::Current(Offset) => self
                .File_system
                .Set_file_position(self.File_identifier, Position_type::Current(Offset))
                .map(|x| x.0)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Error")),
        }
    }
}
