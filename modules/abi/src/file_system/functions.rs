/// This module implements the POSIX like file system C ABI.
use core::{
    cmp::min,
    ffi::{CStr, c_char},
    num::NonZeroU32,
    ptr::copy_nonoverlapping,
};

use futures::block_on;

use file_system::{Error, FileIdentifier, Flags, Mode, Open, Status};
use virtual_file_system::get_instance as get_file_system_instance;

use crate::{XilaTime, context, into_position};

use super::{
    XilaFileSystemMode, XilaFileSystemOpen, XilaFileSystemResult, XilaFileSystemSize,
    XilaFileSystemStatistics, XilaFileSystemStatus, XilaFileSystemWhence, XilaUniqueFileIdentifier,
};

/// This function is used to convert a function returning a Result into a u32.
pub fn into_u32<F>(function: F) -> XilaFileSystemResult
where
    F: FnOnce() -> Result<(), NonZeroU32>,
{
    match function() {
        Ok(()) => 0,
        Err(error) => error.get(),
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
    file: XilaUniqueFileIdentifier,
    statistics: *mut XilaFileSystemStatistics,
) -> XilaFileSystemResult {
    into_u32(move || {
        let task_identifier = context::get_instance().get_current_task_identifier();

        let statistics = XilaFileSystemStatistics::from_mutable_pointer(statistics)
            .ok_or(Error::InvalidParameter)?;

        let file = file_system::UniqueFileIdentifier::from_raw(file);

        *statistics = XilaFileSystemStatistics::from_statistics(
            block_on(get_file_system_instance().get_statistics(file, task_identifier))
                .expect("Failed to get file statistics."),
        );

        Ok(())
    })
}

/// This function is used to get the statistics of a file from its path.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_get_statistics_from_path(
    path: *const c_char,
    statistics: *mut XilaFileSystemStatistics,
    _: bool,
) -> XilaFileSystemResult {
    into_u32(move || {
        let path = CStr::from_ptr(path)
            .to_str()
            .map_err(|_| Error::InvalidParameter)?;

        let statistics = XilaFileSystemStatistics::from_mutable_pointer(statistics)
            .ok_or(Error::InvalidParameter)?;

        *statistics = XilaFileSystemStatistics::from_statistics(block_on(
            get_file_system_instance().get_statistics_from_path(&path),
        )?);

        Ok(())
    })
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
    file: XilaUniqueFileIdentifier,
    mode: *mut XilaFileSystemMode,
) -> XilaFileSystemResult {
    // Debug: Getting file access mode

    into_u32(move || {
        let task_identifier = context::get_instance().get_current_task_identifier();

        if mode.is_null() {
            Err(Error::InvalidParameter)?;
        }

        let file = file_system::UniqueFileIdentifier::from_raw(file);

        mode.write(block_on(get_file_system_instance().get_mode(file, task_identifier))?.as_u8());

        Ok(())
    })
}

/// This function is used to close a file.
///
/// # Errors
///
/// This function may return an error if the file system fails to close the file.
///
#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_close(file: XilaUniqueFileIdentifier) -> XilaFileSystemResult {
    into_u32(move || {
        let task_identifier = context::get_instance().get_current_task_identifier();

        let file = file_system::UniqueFileIdentifier::from_raw(file);

        block_on(get_file_system_instance().close(file, task_identifier))?;

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
    file: XilaUniqueFileIdentifier,
    buffers: *const *const u8,
    buffers_length: *const usize,
    buffer_count: usize,
    written: *mut usize,
) -> XilaFileSystemResult {
    into_u32(move || {
        let task_identifier = context::get_instance().get_current_task_identifier();

        let buffers = core::slice::from_raw_parts(buffers, buffer_count);
        let buffers_length = core::slice::from_raw_parts(buffers_length, buffer_count);

        let mut current_written = 0;

        let file = file_system::UniqueFileIdentifier::from_raw(file);

        for (buffer, length) in buffers.iter().zip(buffers_length.iter()) {
            let buffer_slice = core::slice::from_raw_parts(*buffer, *length);

            current_written += usize::from(block_on(get_file_system_instance().write(
                file,
                buffer_slice,
                task_identifier,
            ))?);
        }

        if !written.is_null() {
            *written = current_written;
        }

        Ok(())
    })
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
    file: XilaUniqueFileIdentifier,
    buffers: *mut *mut u8,
    buffers_length: *mut usize,
    buffer_count: usize,
    read: *mut usize,
) -> XilaFileSystemResult {
    into_u32(move || {
        let task_identifier = context::get_instance().get_current_task_identifier();

        let buffers = core::slice::from_raw_parts_mut(buffers, buffer_count);
        let buffers_length = core::slice::from_raw_parts_mut(buffers_length, buffer_count);

        let mut current_read = 0;

        let file = file_system::UniqueFileIdentifier::from_raw(file);

        for (buffer_pointer, buffer_length) in buffers.iter_mut().zip(buffers_length.iter_mut()) {
            let buffer = core::slice::from_raw_parts_mut(*buffer_pointer, *buffer_length);

            let read = block_on(get_file_system_instance().read(file, buffer, task_identifier))?;

            current_read += usize::from(read);
        }

        if !read.is_null() {
            *read = current_read;
        }

        Ok(())
    })
}

/// This function is used to perform a read operation on a file at a specific position.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_read_at_position_vectored(
    file: XilaUniqueFileIdentifier,
    buffers: *mut *mut u8,
    buffers_length: *mut usize,
    buffer_count: usize,
    position: u64,
    read: *mut usize,
) -> XilaFileSystemResult {
    into_u32(move || {
        let task_identifier = context::get_instance().get_current_task_identifier();

        let buffers = core::slice::from_raw_parts_mut(buffers, buffer_count);
        let buffers_length = core::slice::from_raw_parts_mut(buffers_length, buffer_count);

        let mut current_read = 0;

        let file: file_system::UniqueFileIdentifier =
            file_system::UniqueFileIdentifier::from_raw(file);

        block_on(get_file_system_instance().set_position(
            file,
            &file_system::Position::Start(position),
            task_identifier,
        ))?;

        for (buffer_pointer, buffer_length) in buffers.iter_mut().zip(buffers_length.iter_mut()) {
            let buffer = core::slice::from_raw_parts_mut(*buffer_pointer, *buffer_length);

            let read = block_on(get_file_system_instance().read(file, buffer, task_identifier))?;

            current_read += usize::from(read);
        }

        if !read.is_null() {
            *read = current_read;
        }

        Ok(())
    })
}

/// This function is used to perform a write operation on a file at a specific position.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_write_at_position_vectored(
    file: XilaUniqueFileIdentifier,
    buffers: *const *const u8,
    buffers_length: *const usize,
    buffer_count: usize,
    position: u64,
    written: *mut usize,
) -> XilaFileSystemResult {
    into_u32(move || {
        let task_identifier = context::get_instance().get_current_task_identifier();

        let buffers = core::slice::from_raw_parts(buffers, buffer_count);
        let buffers_length = core::slice::from_raw_parts(buffers_length, buffer_count);

        let mut current_written = 0;

        let file: file_system::UniqueFileIdentifier =
            file_system::UniqueFileIdentifier::from_raw(file);

        block_on(get_file_system_instance().set_position(
            file,
            &file_system::Position::Start(position),
            task_identifier,
        ))?;

        for (buffer, length) in buffers.iter().zip(buffers_length.iter()) {
            let buffer_slice = core::slice::from_raw_parts(*buffer, *length);

            current_written += usize::from(block_on(get_file_system_instance().write(
                file,
                buffer_slice,
                task_identifier,
            ))?);
        }

        if !written.is_null() {
            *written = current_written;
        }

        Ok(())
    })
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
    file: XilaUniqueFileIdentifier,
    is_a_terminal: *mut bool,
) -> XilaFileSystemResult {
    into_u32(move || {
        let task_identifier = context::get_instance().get_current_task_identifier();

        if is_a_terminal.is_null() {
            Err(Error::InvalidParameter)?;
        }

        let file = file_system::UniqueFileIdentifier::from_raw(file);

        *is_a_terminal = block_on(get_file_system_instance().is_a_terminal(file, task_identifier))?;

        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_is_stdin(file: XilaUniqueFileIdentifier) -> bool {
    let file = file_system::UniqueFileIdentifier::from_raw(file);

    let (_, file) = file.split();

    // Debug: Checking if file is stdin

    file == FileIdentifier::STANDARD_IN
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_is_stderr(file: XilaUniqueFileIdentifier) -> bool {
    let file = file_system::UniqueFileIdentifier::from_raw(file);

    let (_, file) = file.split();

    // Debug: Checking if file is stderr

    file == FileIdentifier::STANDARD_ERROR
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_is_stdout(file: XilaUniqueFileIdentifier) -> bool {
    let file = file_system::UniqueFileIdentifier::from_raw(file);

    let (_, file) = file.split();

    // Debug: Checking if file is stdout

    file == FileIdentifier::STANDARD_OUT
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
    file: *mut XilaUniqueFileIdentifier,
) -> XilaFileSystemResult {
    into_u32(move || {
        let path = core::ffi::CStr::from_ptr(path)
            .to_str()
            .map_err(|_| Error::InvalidParameter)?;

        let mode = Mode::from_u8(mode);
        let open = Open::from_u8(open);
        let status = Status::from_u8(status);

        let flags = Flags::new(mode, Some(open), Some(status));

        // Debug: Opening file

        let task = context::get_instance().get_current_task_identifier();

        *file = block_on(get_file_system_instance().open(&path, flags, task))
            .expect("Failed to open file")
            .into_inner();

        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_set_flags(
    _file: XilaUniqueFileIdentifier,
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
    _file: XilaUniqueFileIdentifier,
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

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_flush(
    file: XilaUniqueFileIdentifier,
    _: bool,
) -> XilaFileSystemResult {
    into_u32(move || {
        let task = context::get_instance().get_current_task_identifier();

        let file = file_system::UniqueFileIdentifier::from_raw(file);

        block_on(get_file_system_instance().flush(file, task))?;

        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_create_symbolic_link_at(
    _: XilaUniqueFileIdentifier,
    _: *const c_char,
    _: *const c_char,
) -> XilaFileSystemResult {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_read_link_at(
    _directory: XilaUniqueFileIdentifier,
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
    file: XilaUniqueFileIdentifier,
    offset: i64,
    whence: XilaFileSystemWhence,
    position: *mut XilaFileSystemSize,
) -> XilaFileSystemResult {
    into_u32(move || {
        let task = context::get_instance().get_current_task_identifier();
        let current_position = into_position(whence, offset);

        // Debug: Setting position

        let file = file_system::UniqueFileIdentifier::from_raw(file);

        *position =
            block_on(get_file_system_instance().set_position(file, &current_position, task))?
                .as_u64();

        Ok(())
    })
}

/// This function is used to get the position in a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_create_directory(
    path: *const c_char,
) -> XilaFileSystemResult {
    into_u32(move || {
        let path = CStr::from_ptr(path)
            .to_str()
            .map_err(|_| Error::InvalidParameter)?;

        // Debug: Creating directory

        let task = context::get_instance().get_current_task_identifier();
        block_on(get_file_system_instance().create_directory(&path, task))?;

        Ok(())
    })
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

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_set_times(
    _: XilaUniqueFileIdentifier,
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
    into_u32(|| {
        let path = CStr::from_ptr(_path)
            .to_str()
            .map_err(|_| Error::InvalidParameter)?;

        block_on(get_file_system_instance().remove(path))?;

        Ok(())
    })
}

/// This function is used to truncate a file.
#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_truncate(
    _file: XilaUniqueFileIdentifier,
    _length: XilaFileSystemSize,
) -> XilaFileSystemResult {
    into_u32(move || {
        let _task = context::get_instance().get_current_task_identifier();

        let _file = file_system::UniqueFileIdentifier::from_raw(_file);

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
    _file: XilaUniqueFileIdentifier,
    _offset: XilaFileSystemSize,
    _length: XilaFileSystemSize,
    _advice: u8,
) -> XilaFileSystemResult {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_allocate(
    _file: XilaUniqueFileIdentifier,
    _offset: XilaFileSystemSize,
    _length: XilaFileSystemSize,
) -> XilaFileSystemResult {
    todo!()
}
