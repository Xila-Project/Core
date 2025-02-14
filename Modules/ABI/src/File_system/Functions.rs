/// This module implements the POSIX like file system C ABI.
extern crate alloc;

use core::{
    cmp::min,
    ffi::{c_char, CStr},
    num::NonZeroU32,
    ptr::copy_nonoverlapping,
};

use File_system::{
    Error_type, File_identifier_type, Flags_type, Mode_type, Open_type, Status_type,
};

use Task::Get_instance as Get_task_manager_instance;
use Virtual_file_system::Get_instance as Get_file_system_instance;

use crate::Into_position;

use super::{
    Whence_type, Xila_mode_type, Xila_open_type, Xila_size_type, Xila_statistics_type,
    Xila_status_type, Xila_unique_file_identifier_type,
};

/// This function is used to convert a function returning a Result into a u32.
pub fn Into_u32<F>(Function: F) -> u32
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
    Statistics: *mut Xila_statistics_type,
) -> u32 {
    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Statistics = Xila_statistics_type::From_mutable_pointer(Statistics)
            .ok_or(Error_type::Invalid_parameter)?;

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        *Statistics = Xila_statistics_type::From_statistics(
            Get_file_system_instance()
                .Get_statistics(File, Task_identifier)
                .expect("Failed to get file statistics."),
        );

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
    File: Xila_unique_file_identifier_type,
    Mode: *mut Xila_mode_type,
) -> u32 {
    println!("Getting file access mode : {:?}", File);

    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        if Mode.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        Mode.write(
            Get_file_system_instance()
                .Get_mode(File, Task_identifier)?
                .As_u8(),
        );

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
pub extern "C" fn Xila_file_system_close(File: Xila_unique_file_identifier_type) -> u32 {
    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        Get_file_system_instance().Close(File, Task_identifier)?;

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
) -> u32 {
    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Buffers = std::slice::from_raw_parts(Buffers, Buffer_count);
        let Buffers_length = std::slice::from_raw_parts(Buffers_length, Buffer_count);

        let mut Current_written = 0;

        for (Buffer, Length) in Buffers.iter().zip(Buffers_length.iter()) {
            let Buffer_slice = std::slice::from_raw_parts(*Buffer, *Length);

            let File = File_system::Unique_file_identifier_type::From_raw(File);

            Current_written += usize::from(Get_file_system_instance().Write(
                File,
                Buffer_slice,
                Task_identifier,
            )?);
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
) -> u32 {
    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Buffers = std::slice::from_raw_parts_mut(Buffers, Buffer_count);
        let Buffers_length = std::slice::from_raw_parts_mut(Buffers_length, Buffer_count);

        let mut Current_read = 0;

        for (Buffer_pointer, Buffer_length) in Buffers.iter_mut().zip(Buffers_length.iter_mut()) {
            let Buffer = std::slice::from_raw_parts_mut(*Buffer_pointer, *Buffer_length);

            let File = File_system::Unique_file_identifier_type::From_raw(File);

            let Read = Get_file_system_instance().Read(File, Buffer, Task_identifier)?;

            Current_read += usize::from(Read);
        }

        if !Read.is_null() {
            *Read = Current_read;
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
) -> u32 {
    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        if Is_a_terminal.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        *Is_a_terminal = Get_file_system_instance().Is_a_terminal(File, Task_identifier)?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdin(File: Xila_unique_file_identifier_type) -> bool {
    let File = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = File.Split();

    println!(
        "Checking if file is stdin : {:?} : {:?}",
        File,
        File == File_identifier_type::Standard_in
    );

    File == File_identifier_type::Standard_in
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stderr(File: Xila_unique_file_identifier_type) -> bool {
    let File = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = File.Split();

    println!(
        "Checking if file is stderr : {:?} : {:?}",
        File,
        File == File_identifier_type::Standard_error
    );

    File == File_identifier_type::Standard_error
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdout(File: Xila_unique_file_identifier_type) -> bool {
    let File = File_system::Unique_file_identifier_type::From_raw(File);

    let (_, File) = File.Split();

    println!(
        "Checking if file is stdout : {:?} : {:?}",
        File,
        File == File_identifier_type::Standard_out
    );

    File == File_identifier_type::Standard_out
}

/// This function is used to open a file.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_open(
    Path: *const i8,
    Mode: Xila_mode_type,
    Open: Xila_open_type,
    Status: Xila_status_type,
    File: *mut Xila_unique_file_identifier_type,
) -> u32 {
    Into_u32(move || {
        let Path = std::ffi::CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        let Mode = Mode_type::From_u8(Mode);
        let Open = Open_type::From_u8(Open);
        let Status = Status_type::From_u8(Status);

        let Flags = Flags_type::New(Mode, Some(Open), Some(Status));

        println!("Opening file : {:?} with flags : {:?}", Path, Flags);

        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        *File = Get_file_system_instance()
            .Open(&Path, Flags, Task)
            .expect("Failed to open file")
            .Into_inner();

        Ok(())
    })
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
) -> u32 {
    Into_u32(move || {
        let Path = std::ffi::CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        println!("Resolving path : {:?}", Path);

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
pub extern "C" fn Xila_file_system_flush(File: Xila_unique_file_identifier_type, _: bool) -> u32 {
    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        Get_file_system_instance().Flush(File, Task)?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_read_link_at(
    _Directory: Xila_unique_file_identifier_type,
    _Path: *mut i8,
    _Size: usize,
    _Used: *mut usize,
) -> u32 {
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
    Whence: Whence_type,
    Position: *mut Xila_size_type,
) -> u32 {
    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Current_position = Into_position(Whence, Offset);

        println!(
            "Setting position : {:?} : {:?} : {:?}",
            File, Current_position, Position
        );

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        *Position = Get_file_system_instance()
            .Set_position(File, &Current_position, Task)?
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
pub unsafe extern "C" fn Xila_file_system_create_directory(Path: *const c_char) -> u32 {
    Into_u32(move || {
        let Path = CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        println!("Creating directory : {:?}", Path);

        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        Get_file_system_instance().Create_directory(&Path, Task)?;

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
) -> u32 {
    Into_u32(move || {
        let Old_path = CStr::from_ptr(Old_path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        let New_path = CStr::from_ptr(New_path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        println!("Renaming : {:?} to : {:?}", Old_path, New_path);

        Get_file_system_instance().Rename(&Old_path, &New_path)?;

        Ok(())
    })
}
