use std::fmt::Debug;

use File_system::{Entry_type, Error_type, Path_type, Result_type, Unique_file_identifier_type};
use Task::Task_identifier_type;

use crate::Virtual_file_system_type;

pub struct Directory_type<'a> {
    Directory_identifier: Unique_file_identifier_type,
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Task: Task_identifier_type,
}

impl Debug for Directory_type<'_> {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    pub fn Open<'a>(
        Virtual_file_system: &'a Virtual_file_system_type<'a>,
        Path: impl AsRef<Path_type>,
    ) -> Result_type<Directory_type<'a>> {
        let Task = Task::Get_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Directory_identifier = Virtual_file_system.Open_directory(&Path, Task)?;

        Ok(Directory_type {
            Directory_identifier,
            Virtual_file_system,
            Task,
        })
    }

    pub fn Read(&self) -> Result_type<Option<Entry_type>> {
        self.Virtual_file_system
            .Read_directory(self.Directory_identifier, self.Task)
    }
}

impl Drop for Directory_type<'_> {
    fn drop(&mut self) {
        self.Virtual_file_system
            .Close_directory(self.Directory_identifier, self.Task)
            .unwrap();
    }
}

impl Iterator for Directory_type<'_> {
    type Item = Entry_type;

    fn next(&mut self) -> Option<Self::Item> {
        self.Read().unwrap()
    }
}
