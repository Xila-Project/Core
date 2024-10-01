use std::sync::PoisonError;

use super::littlefs;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error_type {
    // LittleFS errors
    Input_output,        // Error during device operation
    Corrupted,           // Corrupted
    No_Entry,            // No directory entry
    Entry_exists,        // Entry already exists
    Not_directory,       // Entry is not a dir
    Is_directory,        // Entry is a dir
    Directory_not_empty, // Dir is not empty
    Bad_file_number,     // Bad file number
    File_too_large,      // File too large
    Invalid_parameter,   // Invalid parameter
    No_space_left,       // No space left on device
    No_memory,           // No more memory available
    No_attribute,        // No data/attr available
    Name_too_long,       // File name too long
    // Custom errors
    Too_many_open_files, // No file identifier left
    Internal_error,      // Internal error
    Poisoned_lock,       // Poisoned lock
    Invalid_identifier,  // Invalid file identifier
}

pub type Result_type<T> = core::result::Result<T, Error_type>;

pub(crate) fn Convert_result(Error: i32) -> Result_type<u32> {
    match Error {
        littlefs::lfs_error_LFS_ERR_IO => Err(Error_type::Input_output),
        littlefs::lfs_error_LFS_ERR_CORRUPT => Err(Error_type::Corrupted),
        littlefs::lfs_error_LFS_ERR_NOENT => Err(Error_type::No_Entry),
        littlefs::lfs_error_LFS_ERR_EXIST => Err(Error_type::Entry_exists),
        littlefs::lfs_error_LFS_ERR_NOTDIR => Err(Error_type::Not_directory),
        littlefs::lfs_error_LFS_ERR_ISDIR => Err(Error_type::Is_directory),
        littlefs::lfs_error_LFS_ERR_NOTEMPTY => Err(Error_type::Directory_not_empty),
        littlefs::lfs_error_LFS_ERR_BADF => Err(Error_type::Bad_file_number),
        littlefs::lfs_error_LFS_ERR_FBIG => Err(Error_type::File_too_large),
        littlefs::lfs_error_LFS_ERR_INVAL => Err(Error_type::Invalid_parameter),
        littlefs::lfs_error_LFS_ERR_NOSPC => Err(Error_type::No_space_left),
        littlefs::lfs_error_LFS_ERR_NOMEM => Err(Error_type::No_memory),
        littlefs::lfs_error_LFS_ERR_NOATTR => Err(Error_type::No_attribute),
        littlefs::lfs_error_LFS_ERR_NAMETOOLONG => Err(Error_type::Name_too_long),
        _ => {
            if Error >= littlefs::lfs_error_LFS_ERR_OK {
                Ok(Error as u32)
            } else {
                Err(Error_type::Internal_error)
            }
        }
    }
}

impl From<Error_type> for crate::Error_type {
    fn from(Error: Error_type) -> Self {
        match Error {
            Error_type::Input_output => Self::Input_output,
            Error_type::Corrupted => Self::Corrupted,
            Error_type::No_Entry => Self::Not_found,
            Error_type::Entry_exists => Self::Already_exists,
            Error_type::Not_directory => Self::Not_directory,
            Error_type::Is_directory => Self::Is_directory,
            Error_type::Directory_not_empty => Self::Directory_not_empty,
            Error_type::Bad_file_number => Self::Invalid_identifier,
            Error_type::File_too_large => Self::File_too_large,
            Error_type::Invalid_parameter => Self::Invalid_input,
            Error_type::No_space_left => Self::File_system_full,
            Error_type::No_memory => Self::No_memory,
            Error_type::No_attribute => Self::No_attribute,
            Error_type::Name_too_long => Self::Name_too_long,
            Error_type::Too_many_open_files => Self::Too_many_open_files,
            Error_type::Internal_error => Self::Internal_error,
            Error_type::Poisoned_lock => Self::Internal_error,
            Error_type::Invalid_identifier => Self::Invalid_identifier,
        }
    }
}

impl<T> From<PoisonError<T>> for Error_type {
    fn from(_: PoisonError<T>) -> Self {
        Error_type::Poisoned_lock
    }
}
