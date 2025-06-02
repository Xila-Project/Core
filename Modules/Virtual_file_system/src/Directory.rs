use core::fmt::Debug;

use File_system::{Entry_type, Path_type, Result_type, Unique_file_identifier_type};
use Futures::block_on;
use Task::Task_identifier_type;

use crate::Virtual_file_system_type;

pub struct Directory_type<'a> {
    Directory_identifier: Unique_file_identifier_type,
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Task: Task_identifier_type,
}

impl Debug for Directory_type<'_> {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Formatter
            .debug_struct("Directory_type")
            .field("File_identifier", &self.Directory_identifier)
            .field(
                "Virtual_file_system",
                &(self.Virtual_file_system as *const _),
            )
            .finish()
    }
}

impl Directory_type<'_> {
    pub async fn Create<'a>(
        Virtual_file_system: &'a Virtual_file_system_type<'a>,
        Path: impl AsRef<Path_type>,
    ) -> Result_type<()> {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        Virtual_file_system.Create_directory(&Path, Task).await
    }

    pub async fn Open<'a>(
        Virtual_file_system: &'a Virtual_file_system_type<'a>,
        Path: impl AsRef<Path_type>,
    ) -> Result_type<Directory_type<'a>> {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Directory_identifier = Virtual_file_system.Open_directory(&Path, Task).await?;

        Ok(Directory_type {
            Directory_identifier,
            Virtual_file_system,
            Task,
        })
    }

    pub async fn Read(&self) -> Result_type<Option<Entry_type>> {
        self.Virtual_file_system
            .Read_directory(self.Directory_identifier, self.Task)
            .await
    }
}

impl Drop for Directory_type<'_> {
    fn drop(&mut self) {
        block_on(
            self.Virtual_file_system
                .Close_directory(self.Directory_identifier, self.Task),
        )
        .unwrap();
    }
}

impl Iterator for Directory_type<'_> {
    type Item = Entry_type;

    fn next(&mut self) -> Option<Self::Item> {
        block_on(self.Read()).unwrap()
    }
}
