/// This module implements the POSIX like file system C ABI.
use core::{
    cmp::min,
    ffi::{c_char, CStr},
    num::NonZeroU32,
    ptr::copy_nonoverlapping,
};

use Futures::block_on;

use File_system::{
    Error_type, File_identifier_type, Flags_type, Mode_type, Open_type, Status_type,
};
use Virtual_file_system::Get_instance as Get_file_system_instance;

use crate::{Context, Into_position, Xila_time_type};

use super::{
    Xila_file_system_mode_type, Xila_file_system_open_type, Xila_file_system_result_type,
    Xila_file_system_size_type, Xila_file_system_statistics_type, Xila_file_system_status_type,
    Xila_file_system_whence_type, Xila_unique_file_identifier_type,
};

/// This function is used to convert a function returning a Result into a u32.
pub fn Into_u32<F>(Function: F) -> Xila_file_system_result_type
where
    F: FnOnce() -> Result<(), NonZeroU32>,
{
    match Function() {
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
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_get_statistics(
    file: Xila_unique_file_identifier_type,
    statistics: *mut Xila_file_system_statistics_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task_identifier = Context::Get_instance().Get_current_task_identifier();

        let Statistics = Xila_file_system_statistics_type::From_mutable_pointer(statistics)
            .ok_or(Error_type::Invalid_parameter)?;

        let File = File_system::Unique_file_identifier_type::From_raw(file);

        *Statistics = Xila_file_system_statistics_type::from_statistics(
            block_on(Get_file_system_instance().Get_statistics(File, task_identifier))
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
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_get_statistics_from_path(
    path: *const c_char,
    statistics: *mut Xila_file_system_statistics_type,
    _: bool,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let path = CStr::from_ptr(path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        let Statistics = Xila_file_system_statistics_type::From_mutable_pointer(statistics)
            .ok_or(Error_type::Invalid_parameter)?;

        *Statistics = Xila_file_system_statistics_type::from_statistics(block_on(
            Get_file_system_instance().Get_statistics_from_path(&path),
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
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_get_access_mode(
    file: Xila_unique_file_identifier_type,
    mode: *mut Xila_file_system_mode_type,
) -> Xila_file_system_result_type {
    // Debug: Getting file access mode

    Into_u32(move || {
        let task_identifier = Context::Get_instance().Get_current_task_identifier();

        if mode.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let File = File_system::Unique_file_identifier_type::From_raw(file);

        mode.write(block_on(Get_file_system_instance().Get_mode(File, task_identifier))?.As_u8());

        Ok(())
    })
}

/// This function is used to close a file.
///
/// # Errors
///
/// This function may return an error if the file system fails to close the file.
///
#[no_mangle]
pub extern "C" fn Xila_file_system_close(
    file: Xila_unique_file_identifier_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task_identifier = Context::Get_instance().Get_current_task_identifier();

        let File = File_system::Unique_file_identifier_type::From_raw(file);

        block_on(Get_file_system_instance().Close(File, task_identifier))?;

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
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_write_vectored(
    file: Xila_unique_file_identifier_type,
    buffers: *const *const u8,
    buffers_length: *const usize,
    buffer_count: usize,
    written: *mut usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task_identifier = Context::Get_instance().Get_current_task_identifier();

        let Buffers = core::slice::from_raw_parts(buffers, buffer_count);
        let buffers_length = core::slice::from_raw_parts(buffers_length, buffer_count);

        let mut Current_written = 0;

        let File = File_system::Unique_file_identifier_type::From_raw(file);

        for (Buffer, Length) in Buffers.iter().zip(buffers_length.iter()) {
            let buffer_slice = core::slice::from_raw_parts(*Buffer, *Length);

            Current_written += usize::from(block_on(Get_file_system_instance().Write(
                File,
                buffer_slice,
                task_identifier,
            ))?);
        }

        if !written.is_null() {
            *written = Current_written;
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
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_read_vectored(
    file: Xila_unique_file_identifier_type,
    buffers: *mut *mut u8,
    buffers_length: *mut usize,
    buffer_count: usize,
    read: *mut usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task_identifier = Context::Get_instance().Get_current_task_identifier();

        let Buffers = core::slice::from_raw_parts_mut(buffers, buffer_count);
        let buffers_length = core::slice::from_raw_parts_mut(buffers_length, buffer_count);

        let mut Current_read = 0;

        let File = File_system::Unique_file_identifier_type::From_raw(file);

        for (Buffer_pointer, Buffer_length) in Buffers.iter_mut().zip(buffers_length.iter_mut()) {
            let buffer = core::slice::from_raw_parts_mut(*Buffer_pointer, *Buffer_length);

            let Read = block_on(Get_file_system_instance().Read(File, buffer, task_identifier))?;

            Current_read += usize::from(Read);
        }

        if !read.is_null() {
            *read = Current_read;
        }

        Ok(())
    })
}

/// This function is used to perform a read operation on a file at a specific position.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_read_at_position_vectored(
    file: Xila_unique_file_identifier_type,
    buffers: *mut *mut u8,
    buffers_length: *mut usize,
    buffer_count: usize,
    position: u64,
    read: *mut usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task_identifier = Context::Get_instance().Get_current_task_identifier();

        let Buffers = core::slice::from_raw_parts_mut(buffers, buffer_count);
        let buffers_length = core::slice::from_raw_parts_mut(buffers_length, buffer_count);

        let mut Current_read = 0;

        let File: File_system::Unique_file_identifier_type =
            File_system::Unique_file_identifier_type::From_raw(file);

        block_on(Get_file_system_instance().Set_position(
            File,
            &File_system::Position_type::Start(position),
            task_identifier,
        ))?;

        for (Buffer_pointer, Buffer_length) in Buffers.iter_mut().zip(buffers_length.iter_mut()) {
            let buffer = core::slice::from_raw_parts_mut(*Buffer_pointer, *Buffer_length);

            let Read = block_on(Get_file_system_instance().Read(File, buffer, task_identifier))?;

            Current_read += usize::from(Read);
        }

        if !read.is_null() {
            *read = Current_read;
        }

        Ok(())
    })
}

/// This function is used to perform a write operation on a file at a specific position.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_write_at_position_vectored(
    file: Xila_unique_file_identifier_type,
    buffers: *const *const u8,
    buffers_length: *const usize,
    buffer_count: usize,
    position: u64,
    written: *mut usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task_identifier = Context::Get_instance().Get_current_task_identifier();

        let Buffers = core::slice::from_raw_parts(buffers, buffer_count);
        let buffers_length = core::slice::from_raw_parts(buffers_length, buffer_count);

        let mut Current_written = 0;

        let File: File_system::Unique_file_identifier_type =
            File_system::Unique_file_identifier_type::From_raw(file);

        block_on(Get_file_system_instance().Set_position(
            File,
            &File_system::Position_type::Start(position),
            task_identifier,
        ))?;

        for (Buffer, Length) in Buffers.iter().zip(buffers_length.iter()) {
            let buffer_slice = core::slice::from_raw_parts(*Buffer, *Length);

            Current_written += usize::from(block_on(Get_file_system_instance().Write(
                File,
                buffer_slice,
                task_identifier,
            ))?);
        }

        if !written.is_null() {
            *written = Current_written;
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
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_is_a_terminal(
    file: Xila_unique_file_identifier_type,
    is_a_terminal: *mut bool,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task_identifier = Context::Get_instance().Get_current_task_identifier();

        if is_a_terminal.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let File = File_system::Unique_file_identifier_type::From_raw(file);

        *is_a_terminal = block_on(Get_file_system_instance().Is_a_terminal(File, task_identifier))?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdin(File: Xila_unique_file_identifier_type) -> bool {
    let file = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = file.Split();

    // Debug: Checking if file is stdin

    File == File_identifier_type::STANDARD_IN
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stderr(File: Xila_unique_file_identifier_type) -> bool {
    let file = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = file.Split();

    // Debug: Checking if file is stderr

    File == File_identifier_type::STANDARD_ERROR
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdout(File: Xila_unique_file_identifier_type) -> bool {
    let file = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = file.Split();

    // Debug: Checking if file is stdout

    File == File_identifier_type::STANDARD_OUT
}

/// This function is used to open a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_open(
    path: *const c_char,
    mode: Xila_file_system_mode_type,
    open: Xila_file_system_open_type,
    status: Xila_file_system_status_type,
    file: *mut Xila_unique_file_identifier_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let path = core::ffi::CStr::from_ptr(path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        let Mode = Mode_type::From_u8(mode);
        let open = Open_type::From_u8(open);
        let status = Status_type::From_u8(status);

        let Flags = Flags_type::New(Mode, Some(open), Some(status));

        // Debug: Opening file

        let Task = Context::Get_instance().Get_current_task_identifier();

        *file = block_on(Get_file_system_instance().Open(&path, Flags, Task))
            .expect("Failed to open file")
            .Into_inner();

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_set_flags(
    _file: Xila_unique_file_identifier_type,
    _status: Xila_file_system_status_type,
) -> Xila_file_system_result_type {
    todo!()
}

/// This function is used to get the flags of a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_get_flags(
    _file: Xila_unique_file_identifier_type,
    _status: *mut Xila_file_system_status_type,
) -> Xila_file_system_result_type {
    todo!()
}

/// This function is used to convert a path to a resolved path (i.e. a path without symbolic links or relative paths).
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_resolve_path(
    path: *const i8,
    resolved_path: *mut u8,
    resolved_path_size: usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let path = core::ffi::CStr::from_ptr(path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

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

#[no_mangle]
pub extern "C" fn Xila_file_system_flush(
    file: Xila_unique_file_identifier_type,
    _: bool,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task = Context::Get_instance().Get_current_task_identifier();

        let File = File_system::Unique_file_identifier_type::From_raw(file);

        block_on(Get_file_system_instance().Flush(File, task))?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_create_symbolic_link_at(
    _: Xila_unique_file_identifier_type,
    _: *const c_char,
    _: *const c_char,
) -> Xila_file_system_result_type {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_file_system_read_link_at(
    _directory: Xila_unique_file_identifier_type,
    _path: *mut i8,
    _size: usize,
    _used: *mut usize,
) -> Xila_file_system_result_type {
    todo!()
}

/// This function is used to set the position in a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_set_position(
    file: Xila_unique_file_identifier_type,
    offset: i64,
    whence: Xila_file_system_whence_type,
    position: *mut Xila_file_system_size_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let task = Context::Get_instance().Get_current_task_identifier();
        let current_position = Into_position(whence, offset);

        // Debug: Setting position

        let File = File_system::Unique_file_identifier_type::From_raw(file);

        *position =
            block_on(Get_file_system_instance().Set_position(File, &current_position, task))?
                .As_u64();

        Ok(())
    })
}

/// This function is used to get the position in a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_create_directory(
    path: *const c_char,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let path = CStr::from_ptr(path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        // Debug: Creating directory

        let Task = Context::Get_instance().Get_current_task_identifier();
        block_on(Get_file_system_instance().Create_directory(&path, Task))?;

        Ok(())
    })
}

/// This function is used to rename (move) a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_rename(
    old_path: *const c_char,
    new_path: *const c_char,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let old_path = CStr::from_ptr(old_path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        let New_path = CStr::from_ptr(new_path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        // Debug: Renaming files

        block_on(Get_file_system_instance().Rename(&old_path, &New_path))?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_set_times(
    _: Xila_unique_file_identifier_type,
    _: Xila_time_type,
    _: Xila_time_type,
    _: u8,
) -> Xila_file_system_result_type {
    todo!()
}

/// This function is used to set access and modification times of a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_set_times_from_path(
    _path: *const c_char,
    _access: Xila_time_type,
    _modification: Xila_time_type,
    _flags: u8,
    _follow: bool,
) -> Xila_file_system_result_type {
    todo!()
}

/// This function is used to remove a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_remove(
    _path: *const c_char,
) -> Xila_file_system_result_type {
    Into_u32(|| {
        let path = CStr::from_ptr(_path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        block_on(Get_file_system_instance().Remove(path))?;

        Ok(())
    })
}

/// This function is used to truncate a file.
#[no_mangle]
pub extern "C" fn Xila_file_system_truncate(
    _file: Xila_unique_file_identifier_type,
    _length: Xila_file_system_size_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let _task = Context::Get_instance().Get_current_task_identifier();

        let _File = File_system::Unique_file_identifier_type::From_raw(_file);

        todo!();
    })
}

/// This function is used to create a symbolic link.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_link(
    _path: *const c_char,
    _link: *const c_char,
) -> Xila_file_system_result_type {
    todo!()
}

/// This function is used to advice the file system about the access pattern of a file.
#[no_mangle]
pub extern "C" fn Xila_file_system_advise(
    _file: Xila_unique_file_identifier_type,
    _offset: Xila_file_system_size_type,
    _length: Xila_file_system_size_type,
    _advice: u8,
) -> Xila_file_system_result_type {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_file_system_allocate(
    _file: Xila_unique_file_identifier_type,
    _offset: Xila_file_system_size_type,
    _length: Xila_file_system_size_type,
) -> Xila_file_system_result_type {
    todo!()
}
