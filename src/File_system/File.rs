use super::Fundamentals::*;
use std::io::{Read, Write, Seek};

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
    Symbolic_link
}

pub trait File_traits : Read + Write + Seek {
    // - Operations
    fn Set_position(&mut self, Offset: Size_type) -> Result<Size_type, ()> {
        match self.seek(std::io::SeekFrom::Start(Offset.0)) {
            Ok(Position) => Ok(Position.into()),
            Err(_) => Err(()),
        }
    }
    fn Write(&mut self, Buffer: &[u8]) -> Result<(), ()> {
        match self.write_all(Buffer) {
            Ok(Size) => Ok(Size.into()),
            Err(_) => Err(()),
        }
    }
    fn Write_line(&mut self, Buffer: &[u8]) -> Result<(), ()> {
        match self.write_all(Buffer) {
            Ok(Size) => Ok(Size.into()),
            Err(_) => Err(()),
        }
    }
    fn Read(&mut self, Buffer: &mut [u8]) -> Result<(), ()> {
        match self.read_exact(Buffer) {
            Ok(Size) => Ok(Size.into()),
            Err(_) => Err(()),
        }
    }
    fn Read_line(&mut self, Buffer: &mut [u8]) -> Result<(), ()> {
        match self.read_exact(Buffer) {
            Ok(Size) => Ok(Size.into()),
            Err(_) => Err(()),
        }
    }

    fn Get_position(&mut self) -> Size_type {
        match self.seek(std::io::SeekFrom::Current(0)) {
            Ok(Offset) => Offset.into(),
            Err(_) => Size_type(0),
        }
    }

    // - Metadata
    fn Get_size(&self) -> Result<Size_type, ()>;
    fn Get_type(&self) -> Result<Type_type, ()>;

}
