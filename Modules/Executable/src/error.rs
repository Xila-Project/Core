use core::fmt::Display;
use core::result::Result;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, Clone)]
pub enum Error_type {
    File_system(File_system::Error_type),
    Task(Task::Error_type),
    Failed_to_get_main_function,
    Invalid_stack_size,
    Permission_denied,
}

impl Display for Error_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error_type::File_system(error) => write!(formatter, "{error}"),
            Error_type::Task(error) => write!(formatter, "{error}"),
            Error_type::Failed_to_get_main_function => {
                write!(formatter, "Failed to get main function")
            }
            Error_type::Invalid_stack_size => write!(formatter, "Invalid stack size"),
            Error_type::Permission_denied => write!(formatter, "Permission denied"),
        }
    }
}

impl From<File_system::Error_type> for Error_type {
    fn from(error: File_system::Error_type) -> Self {
        Error_type::File_system(error)
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(error: Task::Error_type) -> Self {
        Error_type::Task(error)
    }
}
