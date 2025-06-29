use core::{fmt::Display, num::NonZeroU32};

pub type Result_type<T> = core::result::Result<T, Error_type>;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
#[repr(C)]
pub enum Error_type {
    Failed_to_initialize_file_system = 1,
    Permission_denied,
    Not_found,
    Already_exists,
    Directory_already_exists,
    File_system_full,
    File_system_error,
    Invalid_path,
    Invalid_file,
    Invalid_directory,
    Invalid_symbolic_link,
    Unknown,
    Invalid_identifier,
    Failed_to_get_task_informations,
    Failed_to_get_users_informations,
    Too_many_mounted_file_systems,
    Too_many_open_files,
    Internal_error,
    Invalid_mode,
    Unsupported_operation,
    Ressource_busy,
    Already_initialized,
    Not_initialized,
    Failed_to_get_users_manager_instance,
    Failed_to_get_task_manager_instance,
    Invalid_parameter,
    Invalid_flags,
    Not_directory,
    Is_directory,
    Input_output,
    Directory_not_empty,
    File_too_large,
    No_attribute,
    Name_too_long,
    Corrupted,
    No_memory,
    No_space_left,
    Time_error,
    Invalid_inode,
    Other,
}

impl Error_type {
    pub fn Get_discriminant(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(*self as u32) }
    }
}

impl From<Task::Error_type> for Error_type {
    fn from(_: Task::Error_type) -> Self {
        Error_type::Failed_to_get_task_informations
    }
}

impl From<Users::Error_type> for Error_type {
    fn from(_: Users::Error_type) -> Self {
        Error_type::Failed_to_get_users_informations
    }
}

impl From<Error_type> for NonZeroU32 {
    fn from(Error: Error_type) -> Self {
        Error.Get_discriminant()
    }
}

impl Display for Error_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let String = match self {
            Error_type::Failed_to_initialize_file_system => "Failed to initialize file system",
            Error_type::Permission_denied => "Permission denied",
            Error_type::Not_found => "Not found",
            Error_type::Already_exists => "Already exists",
            Error_type::Directory_already_exists => "Directory already exists",
            Error_type::File_system_full => "File system full",
            Error_type::File_system_error => "File system error",
            Error_type::Invalid_path => "Invalid path",
            Error_type::Invalid_file => "Invalid file",
            Error_type::Invalid_directory => "Invalid directory",
            Error_type::Invalid_symbolic_link => "Invalid symbolic link",
            Error_type::Unknown => "Unknown",
            Error_type::Invalid_identifier => "Invalid identifier",
            Error_type::Failed_to_get_task_informations => "Failed to get task informations",
            Error_type::Failed_to_get_users_informations => "Failed to get users informations",
            Error_type::Too_many_mounted_file_systems => "Too many mounted file systems",
            Error_type::Too_many_open_files => "Too many open files",
            Error_type::Internal_error => "Internal error",
            Error_type::Invalid_mode => "Invalid mode",
            Error_type::Unsupported_operation => "Unsupported operation",
            Error_type::Ressource_busy => "Ressource busy",
            Error_type::Already_initialized => "Already initialized",
            Error_type::Not_initialized => "Not initialized",
            Error_type::Failed_to_get_users_manager_instance => {
                "Failed to get users manager instance"
            }
            Error_type::Failed_to_get_task_manager_instance => {
                "Failed to get task manager instance"
            }
            Error_type::Invalid_parameter => "Invalid parameter",
            Error_type::Invalid_flags => "Invalid flags",
            Error_type::Not_directory => "Not directory",
            Error_type::Is_directory => "Is directory",
            Error_type::Input_output => "Input output",
            Error_type::Directory_not_empty => "Directory not empty",
            Error_type::File_too_large => "File too large",
            Error_type::No_attribute => "No attribute",
            Error_type::Name_too_long => "Name too long",
            Error_type::Corrupted => "Corrupted",
            Error_type::No_memory => "No memory",
            Error_type::No_space_left => "No space left",
            Error_type::Time_error => "Time error",
            Error_type::Invalid_inode => "Invalid inode",
            Error_type::Other => "Other",
        };

        write!(Formatter, "{String}")
    }
}
