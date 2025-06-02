use core::fmt::Debug;

use Task::Task_identifier_type;

use File_system::{
    Flags_type, Path_type, Position_type, Result_type, Size_type, Statistics_type, Status_type,
    Unique_file_identifier_type,
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
    fn fmt(&self, Formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Formatter
            .debug_struct("File_type")
            .field("File_identifier", &self.File_identifier)
            .field("File_system", &(self.File_system as *const _))
            .finish()
    }
}

impl<'a> File_type<'a> {
    pub async fn Open(
        File_system: &'a Virtual_file_system_type<'a>,
        Path: impl AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<Self> {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let File_identifier = File_system.Open(&Path, Flags, Task).await?;

        Ok(File_type {
            File_identifier,
            File_system,
            Task,
        })
    }

    pub async fn Create_unnamed_pipe(
        File_system: &'a Virtual_file_system_type<'a>,
        Size: usize,
        Status: Status_type,
        Task: Task_identifier_type,
    ) -> Result_type<(Self, Self)> {
        let (File_identifier_read, File_identifier_write) =
            File_system.Create_unnamed_pipe(Task, Status, Size).await?;

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
    pub async fn Set_position(&self, Position: &Position_type) -> Result_type<Size_type> {
        self.File_system
            .Set_position(self.Get_file_identifier(), Position, self.Task)
            .await
    }

    // - Getters
    pub const fn Get_file_identifier(&self) -> Unique_file_identifier_type {
        self.File_identifier
    }

    // - Operations

    pub async fn Write(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        self.File_system
            .Write(self.Get_file_identifier(), Buffer, self.Task)
            .await
    }

    pub async fn Write_line(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        let Size = self.Write(Buffer).await? + self.Write(b"\n").await?;
        Ok(Size)
    }

    pub async fn Read(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        self.File_system
            .Read(self.Get_file_identifier(), Buffer, self.Task)
            .await
    }
    pub async fn Read_line(&self, Buffer: &mut [u8]) -> Result_type<()> {
        let mut Index = 0;
        loop {
            let Size: usize = self.Read(&mut Buffer[Index..Index + 1]).await?.into();
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

    pub async fn Read_to_end(&self, Buffer: &mut Vec<u8>) -> Result_type<Size_type> {
        self.File_system
            .Read_to_end(self.Get_file_identifier(), self.Task, Buffer)
            .await
    }

    pub async fn Get_statistics(&self) -> Result_type<Statistics_type> {
        self.File_system
            .Get_statistics(self.Get_file_identifier(), self.Task)
            .await
    }
}

impl Drop for File_type<'_> {
    fn drop(&mut self) {
        let _ = self
            .File_system
            .Close(self.Get_file_identifier(), self.Task);
    }
}
