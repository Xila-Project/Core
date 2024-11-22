use core::result::Result;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
pub enum Error_type {
    File_system(File_system::Error_type),
    Task(Task::Error_type),
    Failed_to_get_main_function,
    Invalid_stack_size,
}

impl From<File_system::Error_type> for Error_type {
    fn from(Error: File_system::Error_type) -> Self {
        Error_type::File_system(Error)
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(Error: Task::Error_type) -> Self {
        Error_type::Task(Error)
    }
}
