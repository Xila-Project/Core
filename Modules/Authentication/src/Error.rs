pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug)]
pub enum Error_type {
    Failed_to_get_current_task_identifier(Task::Error_type),
    Failed_to_read_users_folder(File_system::Error_type),
    Failed_to_get_user_file_path,
    Failed_to_read_user_file(File_system::Error_type),
}

impl From<Task::Error_type> for Error_type {
    fn from(Error: Task::Error_type) -> Self {
        Self::Failed_to_get_current_task_identifier(Error)
    }
}
