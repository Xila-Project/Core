use std::fmt::Debug;

use Task::Task_identifier_type;

use File_system::{
    Error_type, Flags_type, Path_type, Position_type, Result_type, Size_type, Statistics_type,
    Status_type, Unique_file_identifier_type,
};

use super::Virtual_file_system_type;

/// File structure.
///
/// This structure is used to represent a file in the virtual file system.
/// This is a wrapper around the virtual file system file identifier.
pub struct File_type<'a> {
    File_identifier: Unique_file_identifier_type,
    File_system: &'a Virtual_file_system_type<'a>,
    Task: Task_identifier_type,
}

impl Debug for File_type<'_> {
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
        File_system: &'a Virtual_file_system_type<'a>,
        Path: impl AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<Self> {
        let Task = Task::Get_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let File_identifier = File_system.Open(&Path, Flags, Task)?;

        Ok(File_type {
            File_identifier,
            File_system,
            Task,
        })
    }

    pub fn Create_unnamed_pipe(
        File_system: &'a Virtual_file_system_type<'a>,
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

    pub fn Read_to_end(&self, Buffer: &mut Vec<u8>) -> Result_type<Size_type> {
        self.File_system
            .Read_to_end(self.Get_file_identifier(), self.Task, Buffer)
    }

    pub fn Get_statistics(&self) -> Result_type<Statistics_type> {
        self.File_system
            .Get_statistics(self.Get_file_identifier(), self.Task)
    }
}

impl Drop for File_type<'_> {
    fn drop(&mut self) {
        let _ = self
            .File_system
            .Close(self.Get_file_identifier(), self.Task);
    }
}
