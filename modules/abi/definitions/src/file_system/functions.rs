/// This module implements the POSIX like file system C ABI.
use core::{
    cmp::min,
    ffi::{CStr, c_char},
    num::NonZeroU32,
    ptr::copy_nonoverlapping,
};
use file_system::{AccessFlags, CreateFlags, Flags, StateFlags, character_device};
use task::block_on;
use virtual_file_system::{
    Error, SynchronousDirectory, SynchronousFile, get_instance as get_file_system_instance,
};

use crate::{XilaTime, file_system::into_position};

use super::{
    XilaFileIdentifier, XilaFileSystemMode, XilaFileSystemOpen, XilaFileSystemResult,
    XilaFileSystemSize, XilaFileSystemStatistics, XilaFileSystemStatus, XilaFileSystemWhence,
};

use abi_context::{self as context, FileIdentifier};

/// This function is used to convert a function returning a Result into a u32.
pub fn into_u32<F>(function: F) -> XilaFileSystemResult
where
    F: FnOnce() -> Result<(), virtual_file_system::Error>,
{
    match function() {
        Ok(()) => 0,
        Err(error) => {
            let non_zero: NonZeroU32 = error.into();

            log::error!("File system error: {:?} ({})", error, non_zero);

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
pub unsafe extern "C" fn xila_file_system_get_statistics(
    file: XilaFileIdentifier,
    statistics: *mut XilaFileSystemStatistics,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let statistics = XilaFileSystemStatistics::from_mutable_pointer(statistics)
                .ok_or(Error::InvalidParameter)?;

            let context = context::get_instance();

            let s = if let Some(result) = context.perform_operation_on_directory(
                file.try_into()?,
                SynchronousDirectory::get_statistics,
            ) {
                result.inspect_err(|&e| {
                    log::error!(
                        "Performing operation on directory to get statistics: {:?}",
                        e
                    );
                })?
            } else {
                context
                    .perform_operation_on_file(file.try_into()?, SynchronousFile::get_statistics)
                    .ok_or(Error::InvalidParameter)
                    .inspect_err(|&e| {
                        log::error!("Performing operation on file to get statistics: {:?}", e);
                    })??
            };

            *statistics = XilaFileSystemStatistics::from_statistics(s);

            Ok(())
        })
    }
}

/// This function is used to get the statistics of a file from its path.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_get_statistics_from_path_at(
    directory: XilaFileIdentifier,
    path: *const c_char,
    statistics: *mut XilaFileSystemStatistics,
    _: bool,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let path = CStr::from_ptr(path)
                .to_str()
                .map_err(|_| Error::InvalidParameter)?;

            let context = context::get_instance();

            let task = context.get_current_task_identifier();

            let path = context::get_instance()
                .resolve_path(task, directory.try_into()?, path)
                .ok_or(Error::InvalidParameter)?;

            let statistics = XilaFileSystemStatistics::from_mutable_pointer(statistics)
                .ok_or(Error::InvalidParameter)?;

            *statistics = XilaFileSystemStatistics::from_statistics(block_on(
                get_file_system_instance().get_statistics(&path),
            )?);

            Ok(())
        })
    }
}

/// This function is used to get the access mode of a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the file system fails to get the access mode of the file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_get_access_mode(
    file: XilaFileIdentifier,
    mode: *mut XilaFileSystemMode,
) -> XilaFileSystemResult {
    unsafe {
        // Debug: Getting file access mode

        into_u32(move || {
            if mode.is_null() {
                Err(Error::InvalidParameter)?;
            }

            let m = context::get_instance()
                .perform_operation_on_file_or_directory(
                    file.try_into()?,
                    |f| SynchronousFile::get_access(f),
                    |d| SynchronousDirectory::get_access(d),
                )
                .ok_or(Error::InvalidIdentifier)??;

            mode.write(m.bits());

            Ok(())
        })
    }
}

/// This function is used to close a file.
///
/// # Errors
///
/// This function may return an error if the file system fails to close the file.
///
#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_close(file: XilaFileIdentifier) -> XilaFileSystemResult {
    into_u32(move || {
        let file: FileIdentifier = file.try_into()?;

        if !file.is_directory() {
            context::get_instance()
                .remove_file(file)
                .ok_or(Error::InvalidIdentifier)?
                .close(get_file_system_instance())?;
        } else {
            log::warning!("Attempted to close a directory with identifier: {:?}", file);
        }

        Ok(())
    })
}

/// This function is used perform a vectored write operation on a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the file system fails to open the file.
///
/// # Example
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_write_vectored(
    file: XilaFileIdentifier,
    buffers: *const *const u8,
    buffers_length: *const usize,
    buffer_count: usize,
    written: *mut usize,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let buffers = core::slice::from_raw_parts(buffers, buffer_count);
            let buffers_length = core::slice::from_raw_parts(buffers_length, buffer_count);

            let size = context::get_instance()
                .perform_operation_on_file(file.try_into()?, |file| {
                    let mut current_written = 0;

                    for (buffer, length) in buffers.iter().zip(buffers_length.iter()) {
                        let buffer_slice = core::slice::from_raw_parts(*buffer, *length);

                        current_written += file.write(buffer_slice)?;
                    }

                    Ok::<_, virtual_file_system::Error>(current_written)
                })
                .ok_or(Error::InvalidIdentifier)??;

            if !written.is_null() {
                *written = size;
            }

            Ok(())
        })
    }
}

/// This function is used to perform a write operation on a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the file system fails to open the file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_read_vectored(
    file: XilaFileIdentifier,
    buffers: *mut *mut u8,
    buffers_length: *mut usize,
    buffer_count: usize,
    read: *mut usize,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let buffers = core::slice::from_raw_parts_mut(buffers, buffer_count);
            let buffers_length = core::slice::from_raw_parts_mut(buffers_length, buffer_count);

            let current_read = context::get_instance()
                .perform_operation_on_file(file.try_into()?, |file| {
                    let mut current_read = 0;

                    for (buffer_pointer, buffer_length) in
                        buffers.iter_mut().zip(buffers_length.iter_mut())
                    {
                        let buffer =
                            core::slice::from_raw_parts_mut(*buffer_pointer, *buffer_length);

                        let read_count = file.read(buffer)?;

                        current_read += read_count;
                    }

                    Ok::<_, virtual_file_system::Error>(current_read)
                })
                .ok_or(Error::InvalidIdentifier)??;

            if !read.is_null() {
                *read = current_read;
            }

            Ok(())
        })
    }
}

/// This function is used to perform a read operation on a file at a specific position.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_read_at_position_vectored(
    file: XilaFileIdentifier,
    buffers: *mut *mut u8,
    buffers_length: *mut usize,
    buffer_count: usize,
    position: u64,
    read: *mut usize,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let buffers = core::slice::from_raw_parts_mut(buffers, buffer_count);
            let buffers_length = core::slice::from_raw_parts_mut(buffers_length, buffer_count);

            let current_read = context::get_instance()
                .perform_operation_on_file(file.try_into()?, |file| {
                    file.set_position(&file_system::Position::Start(position))?;

                    let mut current_read = 0;

                    for (buffer_pointer, buffer_length) in
                        buffers.iter_mut().zip(buffers_length.iter_mut())
                    {
                        let buffer =
                            core::slice::from_raw_parts_mut(*buffer_pointer, *buffer_length);

                        let read_count = file.read(buffer)?;

                        current_read += read_count;
                    }

                    Ok::<_, virtual_file_system::Error>(current_read)
                })
                .ok_or(Error::InvalidIdentifier)??;

            if !read.is_null() {
                *read = current_read;
            }

            Ok(())
        })
    }
}

/// This function is used to perform a write operation on a file at a specific position.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_write_at_position_vectored(
    file: XilaFileIdentifier,
    buffers: *const *const u8,
    buffers_length: *const usize,
    buffer_count: usize,
    position: u64,
    written: *mut usize,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let buffers = core::slice::from_raw_parts(buffers, buffer_count);
            let buffers_length = core::slice::from_raw_parts(buffers_length, buffer_count);

            let current_written = context::get_instance()
                .perform_operation_on_file(file.try_into()?, |file| {
                    file.set_position(&file_system::Position::Start(position))?;

                    let mut current_written = 0;

                    for (buffer, length) in buffers.iter().zip(buffers_length.iter()) {
                        let buffer_slice = core::slice::from_raw_parts(*buffer, *length);

                        current_written += file.write(buffer_slice)?;
                    }

                    Ok::<_, virtual_file_system::Error>(current_written)
                })
                .ok_or(Error::InvalidIdentifier)??;

            if !written.is_null() {
                *written = current_written;
            }

            Ok(())
        })
    }
}

/// This function is used to check if a file is a terminal.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Errors
///
/// This function may return an error if the file system fails to open the file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_is_a_terminal(
    file: XilaFileIdentifier,
    is_a_terminal: *mut bool,
) -> XilaFileSystemResult {
    into_u32(move || {
        if is_a_terminal.is_null() {
            Err(Error::InvalidParameter)?;
        }

        unsafe {
            *is_a_terminal = context::get_instance()
                .perform_operation_on_file(file.try_into()?, |file| {
                    file.control(character_device::IS_A_TERMINAL, &())
                })
                .ok_or(Error::InvalidIdentifier)??;
        }

        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_is_stdin(file: XilaFileIdentifier) -> bool {
    if let Ok(file) = file.try_into() {
        // Debug: Checking if file is stdin
        FileIdentifier::STANDARD_IN == file
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_is_stderr(file: XilaFileIdentifier) -> bool {
    if let Ok(file) = file.try_into() {
        FileIdentifier::STANDARD_ERROR == file
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_is_stdout(file: XilaFileIdentifier) -> bool {
    if let Ok(file) = file.try_into() {
        FileIdentifier::STANDARD_OUT == file
    } else {
        false
    }
}

/// This function is used to open a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_open(
    path: *const c_char,
    mode: XilaFileSystemMode,
    open: XilaFileSystemOpen,
    status: XilaFileSystemStatus,
    file: *mut XilaFileIdentifier,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let path = core::ffi::CStr::from_ptr(path)
                .to_str()
                .map_err(|_| Error::InvalidParameter)?;

            let mode = AccessFlags::from_bits_truncate(mode);
            let open = CreateFlags::from_bits_truncate(open);
            let status = StateFlags::from_bits_truncate(status);

            let flags = Flags::new(mode, Some(open), Some(status));

            // Debug: Opening file

            let task = context::get_instance().get_current_task_identifier();

            let f = SynchronousFile::open(get_file_system_instance(), task, path, flags)?;

            *file = context::get_instance()
                .insert_file(task, f, None)
                .ok_or(Error::InvalidIdentifier)?
                .into();

            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_set_flags(
    _file: XilaFileIdentifier,
    _status: XilaFileSystemStatus,
) -> XilaFileSystemResult {
    todo!()
}

/// This function is used to get the flags of a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_get_flags(
    _file: XilaFileIdentifier,
    _status: *mut XilaFileSystemStatus,
) -> XilaFileSystemResult {
    todo!()
}

/// This function is used to convert a path to a resolved path (i.e. a path without symbolic links or relative paths).
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_resolve_path(
    path: *const i8,
    resolved_path: *mut u8,
    resolved_path_size: usize,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let path = core::ffi::CStr::from_ptr(path)
                .to_str()
                .map_err(|_| Error::InvalidParameter)?;

            // Debug: Resolving path

            // Copy path to resolved path.
            copy_nonoverlapping(
                path.as_ptr(),
                resolved_path,
                min(resolved_path_size, path.len()),
            );

            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_flush(
    file: XilaFileIdentifier,
    _: bool,
) -> XilaFileSystemResult {
    into_u32(move || {
        context::get_instance()
            .perform_operation_on_file(file.try_into()?, |file| file.flush())
            .ok_or(Error::InvalidIdentifier)??;

        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_create_symbolic_link_at(
    _: XilaFileIdentifier,
    _: *const c_char,
    _: *const c_char,
) -> XilaFileSystemResult {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_read_link_at(
    _directory: XilaFileIdentifier,
    _path: *mut i8,
    _size: usize,
    _used: *mut usize,
) -> XilaFileSystemResult {
    todo!()
}

/// This function is used to set the position in a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_set_position(
    file: XilaFileIdentifier,
    offset: i64,
    whence: XilaFileSystemWhence,
    position: *mut XilaFileSystemSize,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let current_position = into_position(whence, offset);

            // Debug: Setting position

            let result = context::get_instance()
                .perform_operation_on_file(file.try_into()?, |file| {
                    file.set_position(&current_position)
                })
                .ok_or(Error::InvalidIdentifier)??;

            *position = result;

            Ok(())
        })
    }
}

/// This function is used to get the position in a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_create_directory_at(
    directory: XilaFileIdentifier,
    path: *const c_char,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let path = CStr::from_ptr(path)
                .to_str()
                .map_err(|_| Error::InvalidParameter)?;

            let context = context::get_instance();

            let task = context.get_current_task_identifier();

            let path = context
                .resolve_path(task, directory.try_into()?, path)
                .ok_or(Error::InvalidParameter)?;

            let task = context::get_instance().get_current_task_identifier();
            block_on(get_file_system_instance().create_directory(task, &path))?;

            Ok(())
        })
    }
}

/// This function is used to rename (move) a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_rename(
    old_path: *const c_char,
    new_path: *const c_char,
) -> XilaFileSystemResult {
    unsafe {
        into_u32(move || {
            let old_path = CStr::from_ptr(old_path)
                .to_str()
                .map_err(|_| Error::InvalidParameter)?;

            let new_path = CStr::from_ptr(new_path)
                .to_str()
                .map_err(|_| Error::InvalidParameter)?;

            // Debug: Renaming files

            block_on(get_file_system_instance().rename(&old_path, &new_path))?;

            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_set_times(
    _: XilaFileIdentifier,
    _: XilaTime,
    _: XilaTime,
    _: u8,
) -> XilaFileSystemResult {
    todo!()
}

/// This function is used to set access and modification times of a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_set_times_from_path(
    _path: *const c_char,
    _access: XilaTime,
    _modification: XilaTime,
    _flags: u8,
    _follow: bool,
) -> XilaFileSystemResult {
    todo!()
}

/// This function is used to remove a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_remove(_path: *const c_char) -> XilaFileSystemResult {
    unsafe {
        into_u32(|| {
            let path = CStr::from_ptr(_path)
                .to_str()
                .map_err(|_| Error::InvalidParameter)?;

            let task = context::get_instance().get_current_task_identifier();

            block_on(get_file_system_instance().remove(task, path))?;

            Ok(())
        })
    }
}

/// This function is used to truncate a file.
#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_truncate(
    _file: XilaFileIdentifier,
    _length: XilaFileSystemSize,
) -> XilaFileSystemResult {
    into_u32(move || {
        let _task = context::get_instance().get_current_task_identifier();

        let _file: FileIdentifier = _file.try_into()?;

        todo!();
    })
}

/// This function is used to create a symbolic link.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_link(
    _path: *const c_char,
    _link: *const c_char,
) -> XilaFileSystemResult {
    todo!()
}

/// This function is used to advice the file system about the access pattern of a file.
#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_advise(
    _file: XilaFileIdentifier,
    _offset: XilaFileSystemSize,
    _length: XilaFileSystemSize,
    _advice: u8,
) -> XilaFileSystemResult {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_allocate(
    _file: XilaFileIdentifier,
    _offset: XilaFileSystemSize,
    _length: XilaFileSystemSize,
) -> XilaFileSystemResult {
    todo!()
}
