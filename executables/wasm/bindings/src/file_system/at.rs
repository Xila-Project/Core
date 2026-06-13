use core::ffi::c_char;
use core::mem::MaybeUninit;
use core::{ffi::CStr, ptr::NonNull};

use crate::{EnvironmentContext, FileSystemItem, FileVariantKind, XilaFileSystemItem};
use alloc::borrow::ToOwned;
use xila::abi_declarations::{
    XILA_RESULT_OK, XilaFileSystemAccess, XilaFileSystemOpen, XilaFileSystemResult,
    XilaFileSystemState, XilaFileSystemStatistics, xila_file_system_directory_create,
    xila_file_system_directory_open, xila_file_system_file_open,
    xila_file_system_get_statistics_from_path, xila_file_system_remove, xila_file_system_rename,
};
use xila::file_system::PathOwned;
use xila::virtual_file_system::{SynchronousDirectory, SynchronousFile};
use xila::{
    file_system::Path,
    log,
    virtual_file_system::{self, Error},
};

unsafe fn resolve(
    parent: *mut XilaFileSystemItem,
    path: *const c_char,
) -> virtual_file_system::Result<(Option<NonNull<FileSystemItem>>, PathOwned)> {
    unsafe {
        let path = CStr::from_ptr(path)
            .to_str()
            .map_err(|_| Error::InvalidParameter)?;

        log::information!("Resolving path {path:?} in directory {parent:?} (raw path: {path:?})");

        let parent = NonNull::new(parent);

        let r = match parent {
            Some(mut parent_pointer) => {
                let parent_pointer = XilaFileSystemItem::as_real_mut(parent_pointer.as_mut());

                let resolved_path = parent_pointer
                    .resolve(path)
                    .ok_or (Error::InvalidParameter)
                    .inspect_err(|&error| {
                        log::warning!(
                            "Failed to resolve path {path:?} in directory {parent:?} with error: {error:?}"
                        );
                    })?;

                (Some(NonNull::from(parent_pointer)), resolved_path)
            }
            // If no parent is provided, it means we want to resolve the path from the root of the file system.
            None => (None, Path::new(path).to_owned()),
        };

        Ok(r)
    }
}

/// # Safety
/// The caller must ensure that the provided pointer is valid and points to a properly initialized `XilaFileSystemItem` that is a directory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_open_at(
    parent: *mut XilaFileSystemItem,
    path: *const c_char,
    is_directory: bool,
    access: XilaFileSystemAccess,
    open: XilaFileSystemOpen,
    status: XilaFileSystemState,
    out: *mut *mut XilaFileSystemItem,
) -> XilaFileSystemResult {
    unsafe {
        log::information!(
            "Opening file system item at path {path:?} in directory {parent:?} (is_directory: {is_directory}, access: {access:?}, open: {open:?}, status: {status:?})"
        );

        let context = match EnvironmentContext::get() {
            Some(context) => context,
            None => return XilaFileSystemResult::from(Error::InvalidParameter),
        };

        let task = context.get_task();

        log::information!(
            "Successfully retrieved environment context and task for opening file system item at path {path:?} in directory {parent:?}"
        );

        let (parent, resolved_path) = match resolve(parent, path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        log::information!(
            "Successfully resolved path {path:?} to {resolved_path:?} for opening file system item in directory {parent:?}"
        );

        let task = task.into_inner() as _;

        log::information!(
            "Converted task to raw identifier for opening file system item at path {path:?} in directory {parent:?}: {task:?}"
        );

        let resolved_path_raw = resolved_path.as_str() as *const _ as _;

        let item = if is_directory {
            let mut directory = MaybeUninit::<SynchronousDirectory>::uninit();

            let r = xila_file_system_directory_open(
                task,
                resolved_path_raw,
                directory.as_mut_ptr() as *mut _,
            );

            log::information!(
                "Result of opening directory at path {path:?} in directory {parent:?}: {r}"
            );

            if r != XILA_RESULT_OK {
                log::warning!(
                    "Failed to open directory at path {path:?} in directory {parent:?} with error code {r}"
                );
                return r;
            }

            log::information!(
                "Successfully opened directory at path {path:?} in directory {parent:?}"
            );

            let directory = directory.assume_init();

            if path.is_null() {
                return XilaFileSystemResult::from(Error::InvalidParameter);
            }

            log::information!(
                "Successfully opened directory at path {path:?} in directory {parent:?}"
            );

            let path = match CStr::from_ptr(path).to_str() {
                Ok(path) => path,
                Err(_) => return XilaFileSystemResult::from(Error::InvalidParameter),
            };

            log::information!(
                "Successfully opened directory at path {path:?} in directory {parent:?}"
            );

            FileSystemItem::new_directory(directory, resolved_path)
        } else {
            let mut file = MaybeUninit::<SynchronousFile>::uninit();

            let r = xila_file_system_file_open(
                task,
                resolved_path_raw,
                access,
                open,
                status,
                file.as_mut_ptr() as *mut _,
            );

            log::information!(
                "Result of opening file at path {path:?} in directory {parent:?}: {r}"
            );

            if r != XILA_RESULT_OK {
                return r;
            }

            log::information!(
                "Result of opening file at path {path:?} in directory {parent:?}: {r}"
            );

            FileSystemItem::new_file(file.assume_init(), FileVariantKind::Regular)
        };

        log::information!(
            "Successfully opened file system item at path {path:?} in directory {parent:?}: {item:?}"
        );

        *out = item as _;

        XILA_RESULT_OK
    }
}

/// This function is used to get the statistics of a file from its path.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_get_statistics_at(
    directory: *mut XilaFileSystemItem,
    path: *const c_char,
    statistics: *mut XilaFileSystemStatistics,
    _: bool,
) -> XilaFileSystemResult {
    unsafe {
        log::information!(
            "Getting statistics for file system item at path {path:?} in directory {directory:?}"
        );

        let (_, resolved_path) = match resolve(directory, path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        log::information!(
            "Successfully resolved path {path:?} to {resolved_path:?} for getting statistics of file system item in directory {directory:?}"
        );

        let r = xila_file_system_get_statistics_from_path(
            resolved_path.as_str() as *const _ as _,
            statistics,
        );
        log::information!(
            "Result of getting statistics for file system item at path {path:?} in directory {directory:?}: {r}"
        );
        r
    }
}

/// # Safety
/// The caller must ensure that the provided pointer is valid and points to a properly initialized `XilaFileSystemItem` that is a directory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_rename_at(
    first_directory: *mut XilaFileSystemItem,
    first_path: *const c_char,
    second_directory: *mut XilaFileSystemItem,
    second_path: *const c_char,
) -> XilaFileSystemResult {
    log::information!(
        "Renaming file system item from path {first_path:?} in directory {first_directory:?} to path {second_path:?} in directory {second_directory:?}"
    );
    unsafe {
        let (_, first_resolved_path) = match resolve(first_directory, first_path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        let (_, second_resolved_path) = match resolve(second_directory, second_path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        xila_file_system_rename(
            first_resolved_path.as_str() as *const _ as _,
            second_resolved_path.as_str() as *const _ as _,
        )
    }
}

/// # Safety
/// The caller must ensure that the provided pointer is valid and points to a properly initialized `XilaFileSystemItem` that is a directory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_remove_at(
    directory: *mut XilaFileSystemItem,
    path: *const c_char,
) -> XilaFileSystemResult {
    log::information!("Removing file system item at path {path:?} in directory {directory:?}");
    unsafe {
        let (_, resolved_path) = match resolve(directory, path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        let task = match EnvironmentContext::get() {
            Some(context) => context.get_task(),
            None => return XilaFileSystemResult::from(Error::InvalidParameter),
        };

        xila_file_system_remove(task.into(), resolved_path.as_str() as *const _ as _)
    }
}

/// # Safety
/// The caller must ensure that the provided pointer is valid and points to a properly initialized `XilaFileSystemItem` that is a directory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_directory_create_at(
    directory: *mut XilaFileSystemItem,
    path: *const c_char,
) -> XilaFileSystemResult {
    log::information!("Creating directory at path {path:?} in directory {directory:?}");
    unsafe {
        let (_, resolved_path) = match resolve(directory, path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        xila_file_system_directory_create(0, resolved_path.as_str() as *const _ as _)
    }
}

/// # Safety
/// The caller must ensure that the provided pointer is valid and points to a properly initialized `XilaFileSystemItem` that is a directory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_create_symbolic_link_at(
    _: *mut XilaFileSystemItem,
    _: *const c_char,
    _: *const c_char,
) -> XilaFileSystemResult {
    unimplemented!("Creating symbolic links is not supported in Xila")
}

/// # Safety
/// The caller must ensure that the provided pointer is valid and points to a properly initialized `XilaFileSystemItem` that is a directory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_create_link_at(
    _: *mut XilaFileSystemItem,
    _: *const c_char,
    _: *const c_char,
) -> XilaFileSystemResult {
    unimplemented!("Creating hard links is not supported in Xila")
}

/// # Safety
/// The caller must ensure that the provided pointer is valid and points to a properly initialized `XilaFileSystemItem` that is a directory.
#[unsafe(no_mangle)]
pub extern "C" fn __wasm_file_system_read_link_at(
    _: *mut XilaFileSystemItem,
    _: *mut i8,
    _: usize,
    _: *mut usize,
) -> XilaFileSystemResult {
    unimplemented!("Reading symbolic links is not supported in Xila")
}
