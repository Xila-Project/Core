use core::ffi::c_char;
use core::mem::MaybeUninit;
use core::ops::DerefMut;
use core::{ffi::CStr, ptr::NonNull};

use crate::{EnvironmentContext, FileSystemItem, FileVariantKind, XilaFileSystemItem};
use alloc::borrow::ToOwned;
use xila::abi_declarations::{
    XILA_RESULT_OK, XilaFileSystemAccess, XilaFileSystemDirectory, XilaFileSystemFile,
    XilaFileSystemOpen, XilaFileSystemResult, XilaFileSystemState, XilaFileSystemStatistics,
    xila_file_system_directory_create, xila_file_system_directory_open, xila_file_system_file_open,
    xila_file_system_get_statistics_from_path, xila_file_system_remove, xila_file_system_rename,
};
use xila::file_system::PathOwned;
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

        let parent = NonNull::new(parent);

        let r = match parent {
            Some(mut parent_pointer) => {
                let parent_pointer = parent_pointer.as_mut();

                let resolved_path = parent_pointer
                    .resolve(path)
                    .ok_or(Error::InvalidParameter)?;

                (
                    Some(NonNull::from(parent_pointer.deref_mut())),
                    resolved_path,
                )
            }
            // If no parent is provided, it means we want to resolve the path from the root of the file system.
            None => (None, Path::new(path).to_owned()),
        };

        Ok(r)
    }
}

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

        let context = EnvironmentContext::get();

        let task = context.get_task();

        let (parent, resolved_path) = match resolve(parent, path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        let task = task.into_inner() as _;

        let resolved_path = resolved_path.as_str() as *const _ as _;

        let item = if is_directory {
            let mut directory = MaybeUninit::<XilaFileSystemDirectory>::uninit();

            let r = xila_file_system_directory_open(task, resolved_path, directory.as_mut_ptr());

            if r != XILA_RESULT_OK {
                return r;
            }

            let directory = directory.assume_init();

            if path.is_null() {
                return XilaFileSystemResult::from(Error::InvalidParameter);
            }

            let path = match CStr::from_ptr(path).to_str() {
                Ok(path) => path,
                Err(_) => return XilaFileSystemResult::from(Error::InvalidParameter),
            };

            FileSystemItem::new_directory(directory, parent, path)
        } else {
            let mut file = MaybeUninit::<XilaFileSystemFile>::uninit();

            let r = xila_file_system_file_open(
                task,
                resolved_path,
                access,
                open,
                status,
                file.as_mut_ptr(),
            );

            if r == XILA_RESULT_OK {
                return r;
            }

            FileSystemItem::new_file(file.assume_init(), FileVariantKind::Regular)
        };

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

        xila_file_system_get_statistics_from_path(
            resolved_path.as_str() as *const _ as _,
            statistics,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_rename_at(
    first_directory: *mut XilaFileSystemItem,
    first_path: *const c_char,
    second_directory: *mut XilaFileSystemItem,
    second_path: *const c_char,
) -> XilaFileSystemResult {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_remove_at(
    directory: *mut XilaFileSystemItem,
    path: *const c_char,
) -> XilaFileSystemResult {
    unsafe {
        let (_, resolved_path) = match resolve(directory, path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        let task = EnvironmentContext::get().get_task();

        xila_file_system_remove(task.into(), resolved_path.as_str() as *const _ as _)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_directory_create_at(
    directory: *mut XilaFileSystemItem,
    path: *const c_char,
) -> XilaFileSystemResult {
    unsafe {
        let (_, resolved_path) = match resolve(directory, path) {
            Ok((parent, path)) => (parent, path),
            Err(error) => return error.into(),
        };

        xila_file_system_directory_create(0, resolved_path.as_str() as *const _ as _)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_create_symbolic_link_at(
    _: *mut XilaFileSystemItem,
    _: *const c_char,
    _: *const c_char,
) -> XilaFileSystemResult {
    unimplemented!("Creating symbolic links is not supported in Xila")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_create_link_at(
    _: *mut XilaFileSystemItem,
    _: *const c_char,
    _: *const c_char,
) -> XilaFileSystemResult {
    unimplemented!("Creating hard links is not supported in Xila")
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_file_system_read_link_at(
    _: *mut XilaFileSystemItem,
    _: *mut i8,
    _: usize,
    _: *mut usize,
) -> XilaFileSystemResult {
    unimplemented!("Reading symbolic links is not supported in Xila")
}
