use core::{fmt::Display, num::NonZeroU32};

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
#[repr(u32)]
pub enum Error_type {
    Already_initialized = 1,
    Unavailable_driver,
    Invalid_file_system,
    Invalid_parameter,
    Too_many_open_files,
    Failed_to_get_task_informations,
    File_system(File_system::Error_type) = 0xFF,
    Network(Network::Error_type) = 0x200,
}

impl Error_type {
    pub fn get_discriminant(&self) -> NonZeroU32 {
        unsafe { *<*const _>::from(self).cast::<NonZeroU32>() }
    }
}

impl From<Error_type> for NonZeroU32 {
    fn from(value: Error_type) -> Self {
        let discriminant = value.get_discriminant();

        let Offset = match value {
            Error_type::File_system(Error_type) => Error_type.Get_discriminant().get(),
            Error_type::Network(error_type) => error_type.get_discriminant().get() as u32,
            _ => 0,
        };

        discriminant.saturating_add(Offset)
    }
}

impl From<File_system::Error_type> for Error_type {
    fn from(value: File_system::Error_type) -> Self {
        Self::File_system(value)
    }
}

impl From<Network::Error_type> for Error_type {
    fn from(value: Network::Error_type) -> Self {
        Self::Network(value)
    }
}

impl Display for Error_type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error_type::Already_initialized => write!(f, "Already initialized"),
            Error_type::Unavailable_driver => write!(f, "Unavailable driver"),
            Error_type::Invalid_file_system => write!(f, "Invalid file system"),
            Error_type::Invalid_parameter => write!(f, "Invalid parameter"),
            Error_type::Too_many_open_files => write!(f, "Too many open files"),
            Error_type::Failed_to_get_task_informations => {
                write!(f, "Failed to get task informations")
            }
            Error_type::File_system(err) => write!(f, "File system error: {err}"),
            Error_type::Network(err) => write!(f, "Network error: {err}"),
        }
    }
}
