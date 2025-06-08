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

use Task::Get_instance as Get_task_manager_instance;
use Virtual_file_system::Get_instance as Get_file_system_instance;

use crate::{Into_position, Xila_time_type};

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
        Err(Error) => Error.get(),
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
    File: Xila_unique_file_identifier_type,
    Statistics: *mut Xila_file_system_statistics_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task_identifier = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let Statistics = Xila_file_system_statistics_type::From_mutable_pointer(Statistics)
            .ok_or(Error_type::Invalid_parameter)?;

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        *Statistics = Xila_file_system_statistics_type::From_statistics(
            block_on(Get_file_system_instance().Get_statistics(File, Task_identifier))
                .expect("Failed to get file statistics."),
        );

        Ok(())
    })
}

/// This function is used to get the statistics of a file from its path.
#[no_mangle]
pub extern "C" fn Xila_file_system_get_statistics_from_path(
    _Path: *const c_char,
    _Statistics: *mut Xila_file_system_statistics_type,
    _Follow: bool,
) -> Xila_file_system_result_type {
    todo!()
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
    File: Xila_unique_file_identifier_type,
    Mode: *mut Xila_file_system_mode_type,
) -> Xila_file_system_result_type {
    // Debug: Getting file access mode

    Into_u32(move || {
        let Task_identifier = block_on(Get_task_manager_instance().Get_current_task_identifier());

        if Mode.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        Mode.write(block_on(Get_file_system_instance().Get_mode(File, Task_identifier))?.As_u8());

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
    File: Xila_unique_file_identifier_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task_identifier = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        block_on(Get_file_system_instance().Close(File, Task_identifier))?;

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
    File: Xila_unique_file_identifier_type,
    Buffers: *const *const u8,
    Buffers_length: *const usize,
    Buffer_count: usize,
    Written: *mut usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task_identifier = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let Buffers = core::slice::from_raw_parts(Buffers, Buffer_count);
        let Buffers_length = core::slice::from_raw_parts(Buffers_length, Buffer_count);

        let mut Current_written = 0;

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        for (Buffer, Length) in Buffers.iter().zip(Buffers_length.iter()) {
            let Buffer_slice = core::slice::from_raw_parts(*Buffer, *Length);

            Current_written += usize::from(block_on(Get_file_system_instance().Write(
                File,
                Buffer_slice,
                Task_identifier,
            ))?);
        }

        if !Written.is_null() {
            *Written = Current_written;
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
    File: Xila_unique_file_identifier_type,
    Buffers: *mut *mut u8,
    Buffers_length: *mut usize,
    Buffer_count: usize,
    Read: *mut usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task_identifier = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let Buffers = core::slice::from_raw_parts_mut(Buffers, Buffer_count);
        let Buffers_length = core::slice::from_raw_parts_mut(Buffers_length, Buffer_count);

        let mut Current_read = 0;

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        for (Buffer_pointer, Buffer_length) in Buffers.iter_mut().zip(Buffers_length.iter_mut()) {
            let Buffer = core::slice::from_raw_parts_mut(*Buffer_pointer, *Buffer_length);

            let Read = block_on(Get_file_system_instance().Read(File, Buffer, Task_identifier))?;

            Current_read += usize::from(Read);
        }

        if !Read.is_null() {
            *Read = Current_read;
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
    File: Xila_unique_file_identifier_type,
    Buffers: *mut *mut u8,
    Buffers_length: *mut usize,
    Buffer_count: usize,
    Position: u64,
    Read: *mut usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task_identifier = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let Buffers = core::slice::from_raw_parts_mut(Buffers, Buffer_count);
        let Buffers_length = core::slice::from_raw_parts_mut(Buffers_length, Buffer_count);

        let mut Current_read = 0;

        let File: File_system::Unique_file_identifier_type =
            File_system::Unique_file_identifier_type::From_raw(File);

        block_on(Get_file_system_instance().Set_position(
            File,
            &File_system::Position_type::Start(Position),
            Task_identifier,
        ))?;

        for (Buffer_pointer, Buffer_length) in Buffers.iter_mut().zip(Buffers_length.iter_mut()) {
            let Buffer = core::slice::from_raw_parts_mut(*Buffer_pointer, *Buffer_length);

            let Read = block_on(Get_file_system_instance().Read(File, Buffer, Task_identifier))?;

            Current_read += usize::from(Read);
        }

        if !Read.is_null() {
            *Read = Current_read;
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
    File: Xila_unique_file_identifier_type,
    Buffers: *const *const u8,
    Buffers_length: *const usize,
    Buffer_count: usize,
    Position: u64,
    Written: *mut usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task_identifier = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let Buffers = core::slice::from_raw_parts(Buffers, Buffer_count);
        let Buffers_length = core::slice::from_raw_parts(Buffers_length, Buffer_count);

        let mut Current_written = 0;

        let File: File_system::Unique_file_identifier_type =
            File_system::Unique_file_identifier_type::From_raw(File);

        block_on(Get_file_system_instance().Set_position(
            File,
            &File_system::Position_type::Start(Position),
            Task_identifier,
        ))?;

        for (Buffer, Length) in Buffers.iter().zip(Buffers_length.iter()) {
            let Buffer_slice = core::slice::from_raw_parts(*Buffer, *Length);

            Current_written += usize::from(block_on(Get_file_system_instance().Write(
                File,
                Buffer_slice,
                Task_identifier,
            ))?);
        }

        if !Written.is_null() {
            *Written = Current_written;
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
    File: Xila_unique_file_identifier_type,
    Is_a_terminal: *mut bool,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task_identifier = block_on(Get_task_manager_instance().Get_current_task_identifier());

        if Is_a_terminal.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        *Is_a_terminal = block_on(Get_file_system_instance().Is_a_terminal(File, Task_identifier))?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdin(File: Xila_unique_file_identifier_type) -> bool {
    let File = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = File.Split();

    // Debug: Checking if file is stdin

    File == File_identifier_type::Standard_in
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stderr(File: Xila_unique_file_identifier_type) -> bool {
    let File = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = File.Split();

    // Debug: Checking if file is stderr

    File == File_identifier_type::Standard_error
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdout(File: Xila_unique_file_identifier_type) -> bool {
    let File = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = File.Split();

    // Debug: Checking if file is stdout

    File == File_identifier_type::Standard_out
}

/// This function is used to open a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_open(
    Path: *const c_char,
    Mode: Xila_file_system_mode_type,
    Open: Xila_file_system_open_type,
    Status: Xila_file_system_status_type,
    File: *mut Xila_unique_file_identifier_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Path = core::ffi::CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        let Mode = Mode_type::From_u8(Mode);
        let Open = Open_type::From_u8(Open);
        let Status = Status_type::From_u8(Status);

        let Flags = Flags_type::New(Mode, Some(Open), Some(Status));

        // Debug: Opening file

        let Task = block_on(Get_task_manager_instance().Get_current_task_identifier());

        *File = block_on(Get_file_system_instance().Open(&Path, Flags, Task))
            .expect("Failed to open file")
            .Into_inner();

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_set_flags(
    _File: Xila_unique_file_identifier_type,
    _Status: Xila_file_system_status_type,
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
    _File: Xila_unique_file_identifier_type,
    _Status: *mut Xila_file_system_status_type,
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
    Path: *const i8,
    Resolved_path: *mut u8,
    Resolved_path_size: usize,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Path = core::ffi::CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        // Debug: Resolving path

        // Copy path to resolved path.
        copy_nonoverlapping(
            Path.as_ptr(),
            Resolved_path,
            min(Resolved_path_size, Path.len()),
        );

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_flush(
    File: Xila_unique_file_identifier_type,
    _: bool,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task = block_on(Get_task_manager_instance().Get_current_task_identifier());

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        block_on(Get_file_system_instance().Flush(File, Task))?;

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
    _Directory: Xila_unique_file_identifier_type,
    _Path: *mut i8,
    _Size: usize,
    _Used: *mut usize,
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
    File: Xila_unique_file_identifier_type,
    Offset: i64,
    Whence: Xila_file_system_whence_type,
    Position: *mut Xila_file_system_size_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Task = block_on(Get_task_manager_instance().Get_current_task_identifier());
        let Current_position = Into_position(Whence, Offset);

        // Debug: Setting position

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        *Position =
            block_on(Get_file_system_instance().Set_position(File, &Current_position, Task))?
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
    Path: *const c_char,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Path = CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        // Debug: Creating directory

        let Task = block_on(Get_task_manager_instance().Get_current_task_identifier());
        block_on(Get_file_system_instance().Create_directory(&Path, Task))?;

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
    Old_path: *const c_char,
    New_path: *const c_char,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let Old_path = CStr::from_ptr(Old_path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        let New_path = CStr::from_ptr(New_path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        // Debug: Renaming files

        block_on(Get_file_system_instance().Rename(&Old_path, &New_path))?;

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
    _Path: *const c_char,
    _Access: Xila_time_type,
    _Modification: Xila_time_type,
    _Flags: u8,
    _Follow: bool,
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
    _Path: *const c_char,
) -> Xila_file_system_result_type {
    Into_u32(|| {
        let Path = CStr::from_ptr(_Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        block_on(Get_file_system_instance().Remove(Path))?;

        Ok(())
    })
}

/// This function is used to truncate a file.
#[no_mangle]
pub extern "C" fn Xila_file_system_truncate(
    _File: Xila_unique_file_identifier_type,
    _Length: Xila_file_system_size_type,
) -> Xila_file_system_result_type {
    Into_u32(move || {
        let _Task = Get_task_manager_instance().Get_current_task_identifier();

        let _File = File_system::Unique_file_identifier_type::From_raw(_File);

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
    _Path: *const c_char,
    _Link: *const c_char,
) -> Xila_file_system_result_type {
    todo!()
}

/// This function is used to advice the file system about the access pattern of a file.
#[no_mangle]
pub extern "C" fn Xila_file_system_advise(
    _File: Xila_unique_file_identifier_type,
    _Offset: Xila_file_system_size_type,
    _Length: Xila_file_system_size_type,
    _Advice: u8,
) -> Xila_file_system_result_type {
    todo!()
}

#[no_mangle]
pub extern "C" fn Xila_file_system_allocate(
    _File: Xila_unique_file_identifier_type,
    _Offset: Xila_file_system_size_type,
    _Length: Xila_file_system_size_type,
) -> Xila_file_system_result_type {
    todo!()
}
