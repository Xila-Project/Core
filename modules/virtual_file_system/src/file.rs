use core::mem::forget;

use alloc::{string::String, sync::Arc, vec, vec::Vec};
use embedded_io_async::ErrorType;
use file_system::{
    AttributeOperations, Attributes, BaseOperations, BlockDevice, CharacterDevice, Context,
    DirectoryOperations, FileIdentifier, FileSystemOperations, Flags, Path, Position, Size,
    Statistics, Status, UniqueFileIdentifier,
};
use futures::block_on;
use task::TaskIdentifier;

use crate::{Error, Result, pipe::Pipe};

use super::VirtualFileSystem;

enum Item {
    File(Arc<dyn FileSystemOperations>),
    Directory(Arc<dyn FileSystemOperations>),
    BlockDevice(Arc<dyn BlockDevice>),
    CharacterDevice(Arc<dyn CharacterDevice>),
    Pipe(Arc<Pipe>),
}

impl Item {
    fn as_base_operations(&self) -> Option<&dyn BaseOperations> {
        match self {
            Item::File(file_system) => Some(&**file_system),
            Item::BlockDevice(device) => Some(&**device),
            Item::CharacterDevice(device) => Some(&**device),
            Item::Pipe(pipe) => Some(&**pipe),
            _ => None,
        }
    }

    fn as_directory_operations(&self) -> Option<&dyn DirectoryOperations> {
        match self {
            Item::Directory(file_system) => Some(&**file_system),
            _ => None,
        }
    }

    fn as_attributes_operations(&self) -> Option<&dyn AttributeOperations> {
        match self {
            Item::File(fs) => Some(&**fs),
            Item::Directory(fs) => Some(&**fs),
            _ => None,
        }
    }
}

/// File structure.
///
/// This structure is used to represent a file in the virtual file system.
/// This is a wrapper around the virtual file system file identifier.
pub struct File {
    item: Item,
    position: Size,
    flags: Flags,
    context: Context,
}

impl ErrorType for File {
    type Error = file_system::Error;
}

impl embedded_io_async::Write for File<'_> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        File::write(self, buf).await.map(|size| size as usize)
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        File::flush(self).await
    }
}

impl core::fmt::Write for File<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        block_on(self.write(s.as_bytes())).map(|_| ()).map_err(|e| {
            log::error!("Error writing string to file: {}", e);
            core::fmt::Error
        })
    }
}

impl File {
    pub async fn open<'a>(
        file_system: &'a VirtualFileSystem<'a>,
        path: impl AsRef<Path>,
        flags: Flags,
    ) -> Result<Self> {
        let task = task::get_instance().get_current_task_identifier().await;

        let file_identifier = file_system.open(&path, flags, task).await?;

        Ok(File {
            file_identifier,
            file_system,
        })
    }

    pub async fn create_unnamed_pipe(
        file_system: &'a VirtualFileSystem<'a>,
        size: usize,
        status: Status,
        task: TaskIdentifier,
    ) -> Result<(Self, Self)> {
        let (file_identifier_read, file_identifier_write) =
            file_system.create_unnamed_pipe(status, size).await?;

        Ok((
            File {
                file_identifier: file_identifier_read,
                file_system,
            },
            File {
                file_identifier: file_identifier_write,
                file_system,
            },
        ))
    }

    pub async fn set_position(&mut self, position: &Position) -> Result<Size> {
        Ok(self
            .item
            .as_base_operations()
            .ok_or(Error::UnsupportedOperation)?
            .set_position(&mut self.context, position)?)
    }

    // - Operations

    pub async fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        if !self.flags.get_mode().get_write() {
            return Err(Error::InvalidMode);
        }

        let size = self
            .item
            .as_base_operations()
            .ok_or(Error::UnsupportedOperation)?
            .write(&mut self.context, buffer, self.position)?;

        self.position += size as Size;

        Ok(size)
    }

    pub async fn write_line(&mut self, buffer: &[u8]) -> Result<usize> {
        let size = self.write(buffer).await? + self.write(b"\n").await?;

        Ok(size)
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let size = self
            .item
            .as_base_operations()
            .ok_or(Error::UnsupportedOperation)?
            .read(&mut self.context, buffer, self.position)?;

        self.position += size as Size;

        Ok(size)
    }

    pub async fn read_line(&self, buffer: &mut String) -> Result<Size> {
        self.file_system
            .read_line(self.get_file_identifier(), self.buffer)
            .await
    }

    pub async fn read_to_end(&mut self, buffer: &mut Vec<u8>, chunk_size: usize) -> Result<usize> {
        let mut total_read_size = 0;

        let mut chunk = vec![0u8; chunk_size];
        loop {
            let read_size = self
                .item
                .as_base_operations()
                .ok_or(Error::UnsupportedOperation)?
                .read(&mut self.context, &mut chunk, self.position)?;

            if read_size == 0 {
                break;
            }

            self.position += read_size as Size;

            total_read_size += read_size;

            buffer.extend_from_slice(&chunk[..read_size]);

            // Yield to allow other tasks to run.
            //yield_now().await;
        }

        Ok(total_read_size)
    }

    pub async fn get_statistics(&mut self) -> Result<Statistics> {
        let mut attributes = Attributes::default();

        self.item
            .as_attributes_operations()
            .ok_or(Error::UnsupportedOperation)?
            .get_statistics(&mut self.context)
    }

    pub async fn flush(&self) -> Result<()> {
        self.file_system.flush(self.get_file_identifier()).await
    }

    pub async fn close(self) -> crate::Result<()> {
        self.file_system.close(self.get_file_identifier()).await?;

        forget(self); // Prevent Drop from being called

        Ok(())
    }

    pub async fn duplicate(&self) -> Result<Self> {
        let file_identifier = self
            .file_system
            .duplicate(self.get_file_identifier())
            .await?;

        Ok(File {
            file_identifier,
            file_system: self.file_system,
        })
    }

    pub async fn transfer(
        mut self,
        new_task: TaskIdentifier,
        new_file: Option<FileIdentifier>,
    ) -> Result<Self> {
        self.file_identifier = self
            .file_system
            .transfer(self.get_file_identifier(), new_task, new_file)
            .await?;

        Ok(self)
    }

    pub fn into_file_identifier(self) -> UniqueFileIdentifier {
        let file_identifier = self.get_file_identifier();

        forget(self); // Prevent Drop from being called

        file_identifier
    }
}

impl From<&File<'_>> for UniqueFileIdentifier {
    fn from(val: &File<'_>) -> Self {
        val.get_file_identifier()
    }
}

impl Drop for File<'_> {
    fn drop(&mut self) {
        block_on(self.file_system.close(self.get_file_identifier())).unwrap();
    }
}
