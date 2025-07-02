use core::{fmt, num::NonZeroUsize};

#[repr(u8)]
pub enum Error_type {
    Invalid_number_of_arguments = 1,
    Failed_to_get_current_directory,
    Invalid_path,
    Not_a_WASM_file,
    Failed_to_open_file,
    Failed_to_read_file,
    Failed_to_duplicate_standard,
    Failed_to_execute,
}

impl fmt::Display for Error_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let String = match self {
            Error_type::Invalid_number_of_arguments => "Invalid number of arguments",
            Error_type::Failed_to_get_current_directory => "Failed to get current directory",
            Error_type::Invalid_path => "Invalid path",
            Error_type::Not_a_WASM_file => "Not a WASM file",
            Error_type::Failed_to_open_file => "Failed to open file",
            Error_type::Failed_to_read_file => "Failed to read file",
            Error_type::Failed_to_duplicate_standard => "Failed to duplicate standard",
            Error_type::Failed_to_execute => "Failed to execute",
        };

        write!(f, "{String}")
    }
}

impl From<Error_type> for NonZeroUsize {
    fn from(Error: Error_type) -> Self {
        unsafe { NonZeroUsize::new_unchecked(Error as usize) }
    }
}
