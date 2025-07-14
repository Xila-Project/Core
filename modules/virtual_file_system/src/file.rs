use core::fmt::Debug;

use alloc::vec::Vec;
use futures::block_on;
use task::TaskIdentifier;

use file_system::{
    Flags, Path, Position, Result, Size, Statistics_type, Status, UniqueFileIdentifier,
};

use super::VirtualFileSystem;

/// File structure.
///
/// This structure is used to represent a file in the virtual file system.
/// This is a wrapper around the virtual file system file identifier.
pub struct File<'a> {
    file_identifier: UniqueFileIdentifier,
    file_system: &'a VirtualFileSystem<'a>,
    task: TaskIdentifier,
}

impl Debug for File<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("File")
            .field("File_identifier", &self.file_identifier)
            .field("File_system", &(self.file_system as *const _))
            .finish()
    }
}

impl<'a> File<'a> {
    pub async fn open(
        file_system: &'a VirtualFileSystem<'a>,
        path: impl AsRef<Path>,
        flags: Flags,
    ) -> Result<Self> {
        let task = task::get_instance().get_current_task_identifier().await;

        let file_identifier = file_system.open(&path, flags, task).await?;

        Ok(File {
            file_identifier,
            file_system,
            task,
        })
    }

    pub async fn create_unnamed_pipe(
        file_system: &'a VirtualFileSystem<'a>,
        size: usize,
        status: Status,
        task: TaskIdentifier,
    ) -> Result<(Self, Self)> {
        let (file_identifier_read, file_identifier_write) =
            file_system.create_unnamed_pipe(task, status, size).await?;

        Ok((
            File {
                file_identifier: file_identifier_read,
                file_system,
                task,
            },
            File {
                file_identifier: file_identifier_write,
                file_system,
                task,
            },
        ))
    }

    // - Setters
    pub async fn set_position(&self, position: &Position) -> Result<Size> {
        self.file_system
            .set_position(self.get_file_identifier(), position, self.task)
            .await
    }

    // - Getters
    pub const fn get_file_identifier(&self) -> UniqueFileIdentifier {
        self.file_identifier
    }

    // - Operations

    pub async fn write(&self, buffer: &[u8]) -> Result<Size> {
        self.file_system
            .write(self.get_file_identifier(), buffer, self.task)
            .await
    }

    pub async fn write_line(&self, buffer: &[u8]) -> Result<Size> {
        let size = self.write(buffer).await? + self.write(b"\n").await?;
        Ok(size)
    }

    pub async fn read(&self, buffer: &mut [u8]) -> Result<Size> {
        self.file_system
            .read(self.get_file_identifier(), buffer, self.task)
            .await
    }
    pub async fn read_line(&self, buffer: &mut [u8]) -> Result<()> {
        let mut index = 0;
        loop {
            let size: usize = self.read(&mut buffer[index..index + 1]).await?.into();
            if size == 0 {
                break;
            }
            if buffer[index] == b'\n' {
                break;
            }
            index += 1;
        }
        Ok(())
    }

    pub async fn read_to_end(&self, buffer: &mut Vec<u8>) -> Result<Size> {
        self.file_system
            .read_to_end(self.get_file_identifier(), self.task, buffer)
            .await
    }

    pub async fn get_statistics(&self) -> Result<Statistics_type> {
        self.file_system
            .get_statistics(self.get_file_identifier(), self.task)
            .await
    }
}

impl Drop for File<'_> {
    fn drop(&mut self) {
        let _ = block_on(
            self.file_system
                .close(self.get_file_identifier(), self.task),
        );
    }
}
