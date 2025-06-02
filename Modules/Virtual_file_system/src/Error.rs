use core::num::NonZeroU32;

pub type Result_type<T> = Result<T, Error_type>;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
#[repr(u32)]
pub enum Error_type {
    Already_initialized = 1,
    Poisonned_lock,
    Unavailable_driver,
    Invalid_file_system,
    Invalid_parameter,
    Too_many_open_files,
    Failed_to_get_task_informations,
    File_system(File_system::Error_type) = 0xFF,
    Network(Network::Error_type) = 0x200,
}

impl Error_type {
    pub fn Get_discriminant(&self) -> NonZeroU32 {
        unsafe { *<*const _>::from(self).cast::<NonZeroU32>() }
    }
}

impl From<Error_type> for NonZeroU32 {
    fn from(Value: Error_type) -> Self {
        let Discriminant = Value.Get_discriminant();

        let Offset = match Value {
            Error_type::File_system(Error_type) => Error_type.Get_discriminant().get(),
            Error_type::Network(Error_type) => Error_type.Get_discriminant().get() as u32,
            _ => 0,
        };

        Discriminant.saturating_add(Offset)
    }
}

impl From<File_system::Error_type> for Error_type {
    fn from(Value: File_system::Error_type) -> Self {
        Self::File_system(Value)
    }
}

impl From<Network::Error_type> for Error_type {
    fn from(Value: Network::Error_type) -> Self {
        Self::Network(Value)
    }
}
