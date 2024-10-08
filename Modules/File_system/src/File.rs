use std::fmt::Debug;

use Task::Task_identifier_type;

use super::{
    Flags_type, Path_type, Position_type, Result_type, Size_type, Status_type,
    Unique_file_identifier_type, Virtual_file_system::Virtual_file_system_type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Type_type {
    File,
    Directory,
    Block_device,
    Character_device,
    Pipe,
    Socket,
    Symbolic_link,
}

/// File structure.
///
/// This structure is used to represent a file in the virtual file system.
/// This is a wrapper around the virtual file system.
pub struct File_type<'a> {
    File_identifier: Unique_file_identifier_type,
    File_system: &'a Virtual_file_system_type,
    Task: Task_identifier_type,
}

impl<'a> Debug for File_type<'a> {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("File_type")
            .field("File_identifier", &self.File_identifier)
            .field("File_system", &(self.File_system as *const _))
            .finish()
    }
}

impl<'a> File_type<'a> {
    pub fn Open(
        File_system: &'a Virtual_file_system_type,
        Path: impl AsRef<Path_type>,
        Flags: Flags_type,
        Task: Task_identifier_type,
    ) -> Result_type<Self> {
        let File_identifier = File_system.Open(&Path, Flags, Task)?;

        Ok(File_type {
            File_identifier,
            File_system,
            Task,
        })
    }

    pub fn Create_unnamed_pipe(
        File_system: &'a Virtual_file_system_type,
        Size: usize,
        Status: Status_type,
        Task: Task_identifier_type,
    ) -> Result_type<(Self, Self)> {
        let (File_identifier_read, File_identifier_write) =
            File_system.Create_unnamed_pipe(Task, Status, Size)?;

        Ok((
            File_type {
                File_identifier: File_identifier_read,
                File_system,
                Task,
            },
            File_type {
                File_identifier: File_identifier_write,
                File_system,
                Task,
            },
        ))
    }

    // - Setters
    pub fn Set_position(&self, Position: &Position_type) -> Result_type<Size_type> {
        self.File_system
            .Set_position(self.Get_file_identifier(), Position, self.Task)
    }

    // - Getters
    pub const fn Get_file_identifier(&self) -> Unique_file_identifier_type {
        self.File_identifier
    }

    // - Operations

    pub fn Write(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        self.File_system
            .Write(self.Get_file_identifier(), Buffer, self.Task)
    }

    pub fn Write_line(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        let Size = self.Write(Buffer)? + self.Write(b"\n")?;
        Ok(Size)
    }

    pub fn Read(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        self.File_system
            .Read(self.Get_file_identifier(), Buffer, self.Task)
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

impl<'a> Drop for File_type<'a> {
    fn drop(&mut self) {
        let _ = self
            .File_system
            .Close(self.Get_file_identifier(), self.Task);
    }
}
