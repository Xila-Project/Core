use core::ffi::c_char;
use core::mem::MaybeUninit;
use core::ops::DerefMut;
use core::{ffi::CStr, num::NonZeroU32, ptr::NonNull};

use crate::host::bindings::common::{FileSystemIdentifier, global_context};
use crate::host::bindings::file_system::{FileSystemItem, XilaFileSystemItem};
use alloc::borrow::ToOwned;
use smol_str::SmolStr;
use xila::abi_declarations::{
    XILA_FILE_SYSTEM_RESULT_SUCCESS, XilaFileSystemAccess, XilaFileSystemDirectory,
    XilaFileSystemFile, XilaFileSystemOpen, XilaFileSystemResult, XilaFileSystemState,
    XilaFileSystemStatistics, xila_file_system_directory_get_statistics,
    xila_file_system_directory_open, xila_file_system_file_get_statistics,
    xila_file_system_file_open, xila_file_system_get_statistics_from_path, xila_file_system_rename,
};
use xila::file_system::PathOwned;
use xila::{
    file_system::{self, Path},
    log,
    virtual_file_system::{self, Error, SynchronousFile},
};

/// This function is used to convert a function returning a Result into a u32.
pub fn into_result<F>(function: F) -> XilaFileSystemResult
where
    F: FnOnce() -> Result<(), virtual_file_system::Error>,
{
    match function() {
        Ok(()) => XILA_FILE_SYSTEM_RESULT_SUCCESS,
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
                xila_file_system_directory_get_statistics(&mut directory.directory as _, statistics)
            }
            FileSystemItem::File(file) => {
                xila_file_system_file_get_statistics(&mut file.file as _, statistics)
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_get_access(
    item: *mut XilaFileSystemItem,
    access: *mut XilaFileSystemAccess,
) -> XilaFileSystemResult {
    unsafe {
        log::information!("Getting access flags for file system item {item:?}");

        match (*item).deref_mut() {
            FileSystemItem::Directory(directory) => {
                xila_file_system_directory_get_access(&mut directory.directory as _, access)
            }
            FileSystemItem::File(file) => {
                xila_file_system_file_get_access(&mut file.file as _, access)
            }
            _ => virtual_file_system::Error::InvalidParameter.into(),
        }
    }
}
