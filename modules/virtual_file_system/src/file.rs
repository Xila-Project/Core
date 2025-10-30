use core::fmt::Debug;

use alloc::{string::String, vec::Vec};
use embedded_io_async::ErrorType;
use exported_file_system::FileIdentifier;
use file_system::{
    Flags, Path, Position, Result, Size, Statistics_type, Status, UniqueFileIdentifier,
};
use futures::block_on;
use task::TaskIdentifier;

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
            .field("file_identifier", &self.file_identifier)
            .field("file_system", &(self.file_system as *const _))
            .field("task", &self.task)
            .finish()
    }
}

impl ErrorType for File<'_> {
    type Error = file_system::Error;
}

impl embedded_io_async::Write for File<'_> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        File::write(self, buf)
            .await
            .map(|size| size.as_u64() as usize)
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        File::flush(self).await
    }
}

impl core::fmt::Write for File<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        block_on(self.write(s.as_bytes()))
            .map(|_| ())
            .map_err(|_| core::fmt::Error)
    }
}

impl<'a> File<'a> {
    pub fn from(
        file_identifier: UniqueFileIdentifier,
        file_system: &'a VirtualFileSystem<'a>,
        task: TaskIdentifier,
    ) -> Self {
        Self {
            file_identifier,
            file_system,
            task,
        }
    }

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
    pub async fn read_line(&self, buffer: &mut String) -> Result<Size> {
        self.file_system
            .read_line(self.get_file_identifier(), self.task, buffer)
            .await
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

    pub async fn flush(&self) -> Result<()> {
        self.file_system
            .flush(self.get_file_identifier(), self.task)
            .await
    }

    pub async fn duplicate(&self) -> Result<Self> {
        let file_identifier = self
            .file_system
            .duplicate_file_identifier(self.get_file_identifier(), self.task)
            .await?;

        Ok(File {
            file_identifier,
            file_system: self.file_system,
            task: self.task,
        })
    }

    pub async fn transfer(
        mut self,
        new_task: TaskIdentifier,
        new_file: Option<FileIdentifier>,
    ) -> Result<Self> {
        let new_identifier = self
            .file_system
            .transfer(self.get_file_identifier(), self.task, new_task, new_file)
            .await?;

        self.file_identifier = new_identifier;

        Ok(self)
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
