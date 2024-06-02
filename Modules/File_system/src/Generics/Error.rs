use std::num::NonZeroU32;

use Shared::Error_discriminant_trait;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Error_type {
    Failed_to_initialize_file_system = 1,
    Permission_denied,
    File_not_found,
    Directory_not_found,
    File_already_exists,
    Directory_already_exists,
    File_system_full,
    File_system_error,
    Invalid_path,
    Invalid_file,
    Invalid_directory,
    Invalid_symbolic_link,
    Unknown,
    Invalid_file_identifier,
}

impl Error_discriminant_trait for Error_type {
    fn Get_discriminant(&self) -> NonZeroU32 {
        NonZeroU32::new(*self as u32).unwrap()
    }

    fn From_discriminant(Discriminant: NonZeroU32) -> Self {
        unsafe { std::mem::transmute(Discriminant.get() as u8) }
    }
}
