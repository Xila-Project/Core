use super::{
    Flags_type, Path_type, Position_type, Result, Size_type, Status_type,
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
    File_system: Virtual_file_system_type,
}

impl File_type {
    pub fn Open(
        File_system: &Virtual_file_system_type,
        Path: impl AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result<Self> {
        let File_identifier = File_system.Open(Path, Flags)?;

        Ok(File_type {
            File_identifier,
            File_system: File_system.clone(),
        })
    }

    pub fn Create_unamed_pipe(
        File_system: &Virtual_file_system_type,
        Size: usize,
        Status: Status_type,
    ) -> Result<(Self, Self)> {
        let (File_identifier_read, File_identifier_write) =
            File_system.Create_unnamed_pipe(Size, Status)?;

        Ok((
            File_type {
                File_identifier: File_identifier_read,
                File_system: File_system.clone(),
            },
            File_type {
                File_identifier: File_identifier_write,
                File_system: File_system.clone(),
            },
        ))
    }

    // - Setters
    pub fn Set_position(&self, Position: &Position_type) -> Result<Size_type> {
        self.File_system
            .Set_position(self.Get_file_identifier(), Position)
    }

    // - Getters
    pub const fn Get_file_identifier(&self) -> Unique_file_identifier_type {
        self.File_identifier
    }

    // - Operations

    pub fn Write(&self, Buffer: &[u8]) -> Result<Size_type> {
        self.File_system.Write(self.Get_file_identifier(), Buffer)
    }

    pub fn Write_line(&self, Buffer: &[u8]) -> Result<Size_type> {
        let Size = self.Write(Buffer)? + self.Write(b"\n")?;
        Ok(Size)
    }

    pub fn Read(&self, Buffer: &mut [u8]) -> Result<Size_type> {
        self.File_system.Read(self.Get_file_identifier(), Buffer)
    }
    pub fn Read_line(&self, Buffer: &mut [u8]) -> Result<()> {
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
