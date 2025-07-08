use core::fmt::Debug;

use File_system::{Entry_type, Path_type, Result_type, Unique_file_identifier_type};
use Futures::block_on;
use Task::Task_identifier_type;

use crate::Virtual_file_system_type;

pub struct Directory_type<'a> {
    directory_identifier: Unique_file_identifier_type,
    virtual_file_system: &'a Virtual_file_system_type<'a>,
    task: Task_identifier_type,
}

impl Debug for Directory_type<'_> {
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

impl Directory_type<'_> {
    pub async fn create<'a>(
        virtual_file_system: &'a Virtual_file_system_type<'a>,
        path: impl AsRef<Path_type>,
    ) -> Result_type<()> {
        let task = Task::get_instance().get_current_task_identifier().await;

        virtual_file_system.create_directory(&path, task).await
    }

    pub async fn open<'a>(
        virtual_file_system: &'a Virtual_file_system_type<'a>,
        path: impl AsRef<Path_type>,
    ) -> Result_type<Directory_type<'a>> {
        let task = Task::get_instance().get_current_task_identifier().await;

        let Directory_identifier = virtual_file_system.open_directory(&path, task).await?;

        Ok(Directory_type {
            directory_identifier: Directory_identifier,
            virtual_file_system,
            task,
        })
    }

    pub async fn read(&self) -> Result_type<Option<Entry_type>> {
        self.virtual_file_system
            .read_directory(self.directory_identifier, self.task)
            .await
    }
}

impl Drop for Directory_type<'_> {
    fn drop(&mut self) {
        block_on(
            self.virtual_file_system
                .close_directory(self.directory_identifier, self.task),
        )
        .unwrap();
    }
}

impl Iterator for Directory_type<'_> {
    type Item = Entry_type;

    fn next(&mut self) -> Option<Self::Item> {
        block_on(self.read()).unwrap()
    }
}
