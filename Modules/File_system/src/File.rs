use std::fmt::Debug;

use super::{
    Flags_type, Path_type, Position_type, Result_type, Size_type, Status_type,
    Unique_file_identifier_type, Virtual_file_system::Virtual_file_system_type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Type_type {
    File = 1,
    Directory,
    Block_device,
    Character_device,
    Named_pipe,
    Symbolic_link,
}

pub struct File_type {
    File_identifier: Unique_file_identifier_type,
    File_system: &'static Virtual_file_system_type,
}

impl Debug for File_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("File_type")
            .field("File_identifier", &self.File_identifier)
            .field("File_system", &(self.File_system as *const _))
            .finish()
    }
}

impl File_type {
    pub fn Open(
        File_system: &'static Virtual_file_system_type,
        Path: impl AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<Self> {
        let File_identifier = File_system.Open(Path, Flags)?;

        Ok(File_type {
            File_identifier,
            File_system,
        })
    }

    pub fn Create_unnamed_pipe(
        File_system: &'static Virtual_file_system_type,
        Size: Size_type,
        Status: Status_type,
    ) -> Result_type<(Self, Self)> {
        let (File_identifier_read, File_identifier_write) =
            File_system.Create_unnamed_pipe(Size, Status)?;

        Ok((
            File_type {
                File_identifier: File_identifier_read,
                File_system,
            },
            File_type {
                File_identifier: File_identifier_write,
                File_system,
            },
        ))
    }

    // - Setters
    pub fn Set_position(&self, Position: &Position_type) -> Result_type<Size_type> {
        self.File_system
            .Set_position(self.Get_file_identifier(), Position)
    }

    // - Getters
    pub const fn Get_file_identifier(&self) -> Unique_file_identifier_type {
        self.File_identifier
    }

    // - Operations

    pub fn Write(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        self.File_system.Write(self.Get_file_identifier(), Buffer)
    }

    pub fn Write_line(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        let Size = self.Write(Buffer)? + self.Write(b"\n")?;
        Ok(Size)
    }

    pub fn Read(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        self.File_system.Read(self.Get_file_identifier(), Buffer)
    }
    pub fn Read_line(&self, Buffer: &mut [u8]) -> Result_type<()> {
        let mut Index = 0;
        loop {
            let Size: usize = self.Read(&mut Buffer[Index..Index + 1])?.into();
            if Size == 0 {
                break;
            }
            if Buffer[Index] == b'\n' {
                break;
            }
            Index += 1;
        }
        Ok(())
    }
}

impl Drop for File_type {
    fn drop(&mut self) {
        let _ = self.File_system.Close(self.Get_file_identifier());
    }
}
