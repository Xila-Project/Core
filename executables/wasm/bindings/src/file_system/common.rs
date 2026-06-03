use core::num::NonZeroU32;
use core::ops::DerefMut;

use crate::{FileSystemItem, XilaFileSystemItem};
use xila::abi_declarations::{
    XILA_RESULT_OK, XilaFileSystemAccess, XilaFileSystemResult, XilaFileSystemStatistics,
    xila_file_system_directory_get_access, xila_file_system_directory_get_statistics,
    xila_file_system_file_get_access_flags, xila_file_system_file_get_statistics,
};
use xila::{
    file_system::{self},
    log,
    virtual_file_system::{self, Error},
};

/// This function is used to convert a function returning a Result into a u32.
pub fn into_result<F>(function: F) -> XilaFileSystemResult
where
    F: FnOnce() -> Result<(), virtual_file_system::Error>,
{
    match function() {
        Ok(()) => XILA_RESULT_OK,
        Err(error) => {
            let non_zero: NonZeroU32 = error.into();

            if matches!(
                error,
                Error::RessourceBusy | Error::FileSystem(file_system::Error::RessourceBusy)
            ) {
                log::debug!(
                    "File system busy (expected while polling): {:?} ({})",
                    error,
                    non_zero
                );
            } else {
                log::error!("File system error: {:?} ({})", error, non_zero);
            }

            //panic!("File system error: {:?} ({})", error, non_zero.get());

            non_zero.get()
        }
    }
}

/// This function is used to open a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the file system fails to open the file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_get_statistics(
    file: *mut XilaFileSystemItem,
    statistics: *mut XilaFileSystemStatistics,
) -> XilaFileSystemResult {
    unsafe {
        log::information!("Getting statistics for file system item {file:?}");

        match (*file).deref_mut() {
            FileSystemItem::Directory(directory) => {
                xila_file_system_directory_get_statistics(directory.as_mut(), statistics)
            }
            FileSystemItem::File(file) => {
                xila_file_system_file_get_statistics(file.as_mut(), statistics)
            }
        }
    }
}

/// # Safety
/// The caller must ensure that the provided pointer is valid and points to a properly initialized `XilaFileSystemItem` that is a file or a directory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_get_access(
    item: *mut XilaFileSystemItem,
    access: *mut XilaFileSystemAccess,
) -> XilaFileSystemResult {
    unsafe {
        log::information!("Getting access flags for file system item {item:?}");

        let r = match (*item).deref_mut() {
            FileSystemItem::Directory(directory) => {
                xila_file_system_directory_get_access(directory.as_mut(), access)
            }
            FileSystemItem::File(file) => {
                xila_file_system_file_get_access_flags(file.as_mut(), access)
            }
        };

        log::information!("Result of getting access flags for file system item {item:?}: {r}");
        r
    }
}
