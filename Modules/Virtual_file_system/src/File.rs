use core::fmt::Debug;

use alloc::vec::Vec;
use Futures::block_on;
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
    file_identifier: Unique_file_identifier_type,
    file_system: &'a Virtual_file_system_type<'a>,
    task: Task_identifier_type,
}

impl Debug for File_type<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("File_type")
            .field("File_identifier", &self.file_identifier)
            .field("File_system", &(self.file_system as *const _))
            .finish()
    }
}

impl<'a> File_type<'a> {
    pub async fn open(
        file_system: &'a Virtual_file_system_type<'a>,
        path: impl AsRef<Path_type>,
        flags: Flags_type,
    ) -> Result_type<Self> {
        let task = Task::get_instance().get_current_task_identifier().await;

        let File_identifier = file_system.open(&path, flags, task).await?;

        Ok(File_type {
            file_identifier: File_identifier,
            file_system,
            task,
        })
    }

    pub async fn create_unnamed_pipe(
        file_system: &'a Virtual_file_system_type<'a>,
        size: usize,
        status: Status_type,
        task: Task_identifier_type,
    ) -> Result_type<(Self, Self)> {
        let (file_identifier_read, file_identifier_write) =
            file_system.create_unnamed_pipe(task, status, size).await?;

        Ok((
            File_type {
                file_identifier: file_identifier_read,
                file_system,
                task,
            },
            File_type {
                file_identifier: file_identifier_write,
                file_system,
                task,
            },
        ))
    }

    // - Setters
    pub async fn set_position(&self, Position: &Position_type) -> Result_type<Size_type> {
        self.file_system
            .set_position(self.get_file_identifier(), Position, self.task)
            .await
    }

    // - Getters
    pub const fn get_file_identifier(&self) -> Unique_file_identifier_type {
        self.file_identifier
    }

    // - Operations

    pub async fn write(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        self.file_system
            .write(self.get_file_identifier(), Buffer, self.task)
            .await
    }

    pub async fn write_line(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        let size = self.write(Buffer).await? + self.write(b"\n").await?;
        Ok(size)
    }

    pub async fn read(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        self.file_system
            .read(self.get_file_identifier(), Buffer, self.task)
            .await
    }
    pub async fn read_line(&self, Buffer: &mut [u8]) -> Result_type<()> {
        let mut index = 0;
        loop {
            let Size: usize = self.read(&mut Buffer[index..index + 1]).await?.into();
            if Size == 0 {
                break;
            }
            if Buffer[index] == b'\n' {
                break;
            }
            index += 1;
        }
        Ok(())
    }

    pub async fn read_to_end(&self, Buffer: &mut Vec<u8>) -> Result_type<Size_type> {
        self.file_system
            .read_to_end(self.get_file_identifier(), self.task, Buffer)
            .await
    }

    pub async fn get_statistics(&self) -> Result_type<Statistics_type> {
        self.file_system
            .get_statistics(self.get_file_identifier(), self.task)
            .await
    }
}

impl Drop for File_type<'_> {
    fn drop(&mut self) {
        let _ = block_on(
            self.file_system
                .close(self.get_file_identifier(), self.task),
        );
    }
}
