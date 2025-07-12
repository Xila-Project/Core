use core::fmt::Debug;

use file_system::{Entry, Path, Result, UniqueFileIdentifier};
use futures::block_on;
use task::TaskIdentifier;

use crate::VirtualFileSystemType;

pub struct DirectoryType<'a> {
    directory_identifier: UniqueFileIdentifier,
    virtual_file_system: &'a VirtualFileSystemType<'a>,
    task: TaskIdentifier,
}

impl Debug for DirectoryType<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Directory_type")
            .field("File_identifier", &self.directory_identifier)
            .field(
                "Virtual_file_system",
                &(self.virtual_file_system as *const _),
            )
            .finish()
    }
}

impl DirectoryType<'_> {
    pub async fn create<'a>(
        virtual_file_system: &'a VirtualFileSystemType<'a>,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let task = task::get_instance().get_current_task_identifier().await;

        virtual_file_system.create_directory(&path, task).await
    }

    pub async fn open<'a>(
        virtual_file_system: &'a VirtualFileSystemType<'a>,
        path: impl AsRef<Path>,
    ) -> Result<DirectoryType<'a>> {
        let task = task::get_instance().get_current_task_identifier().await;

        let directory_identifier = virtual_file_system.open_directory(&path, task).await?;

        Ok(DirectoryType {
            directory_identifier,
            virtual_file_system,
            task,
        })
    }

    pub async fn read(&self) -> Result<Option<Entry>> {
        self.virtual_file_system
            .read_directory(self.directory_identifier, self.task)
            .await
    }
}

impl Drop for DirectoryType<'_> {
    fn drop(&mut self) {
        block_on(
            self.virtual_file_system
                .close_directory(self.directory_identifier, self.task),
        )
        .unwrap();
    }
}

impl Iterator for DirectoryType<'_> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        block_on(self.read()).unwrap()
    }
}
