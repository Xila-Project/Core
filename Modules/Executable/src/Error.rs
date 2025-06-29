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
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error_type::File_system(Error) => write!(Formatter, "{Error}"),
            Error_type::Task(Error) => write!(Formatter, "{Error}"),
            Error_type::Failed_to_get_main_function => {
                write!(Formatter, "Failed to get main function")
            }
            Error_type::Invalid_stack_size => write!(Formatter, "Invalid stack size"),
            Error_type::Permission_denied => write!(Formatter, "Permission denied"),
        }
    }
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
