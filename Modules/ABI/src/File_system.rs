/// This module implements the POSIX like file system C ABI.
use std::{
    cmp::min,
    ffi::{c_char, CStr, CString},
    num::NonZeroU32,
    ptr::{copy_nonoverlapping, null_mut},
};

use File_system::{
    Error_type, File_identifier_type, Flags_type, Get_instance as Get_file_system_instance,
    Inode_type, Mode_type, Open_type, Position_type, Result_type, Size_type, Statistics_type,
    Status_type, Type_type, Unique_file_identifier_type, Whence_type,
};

use Task::Get_instance as Get_task_manager_instance;

#[no_mangle]
pub static Xila_file_system_mode_read_mask: u8 = Mode_type::Read_bit;
#[no_mangle]
pub static Xila_file_system_mode_write_mask: u8 = Mode_type::Write_bit;

#[no_mangle]
pub static Xila_file_system_open_create_mask: u8 = Open_type::Create_mask;
#[no_mangle]
pub static Xila_file_system_open_create_only_mask: u8 = Open_type::Exclusive_mask;
#[no_mangle]
pub static Xila_file_system_open_truncate_mask: u8 = Open_type::Truncate_mask;

#[no_mangle]
pub static Xila_file_system_status_append_mask: u8 = Status_type::Append_bit;
#[no_mangle]
pub static Xila_file_system_status_non_blocking_mask: u8 = Status_type::Non_blocking_bit;
#[no_mangle]
pub static Xila_file_system_status_synchronous_mask: u8 = Status_type::Synchronous_bit;
#[no_mangle]
pub static Xila_file_system_status_synchronous_data_only_mask: u8 =
    Status_type::Synchronous_data_only_bit;

/// This function is used to convert a function returning a Result into a u32.
fn Into_u32<F>(Function: F) -> u32
where
    F: FnOnce() -> Result_type<()>,
{
    match Function() {
        Ok(()) => 0,
        Err(Error) => NonZeroU32::from(Error).get(),
    }
}

fn Statistics_from_mutable_pointer(
    Pointer: *mut Statistics_type,
) -> Option<&'static mut Statistics_type> {
    if Pointer.is_null() {
        return None;
    }

    if Pointer as usize % std::mem::align_of::<Statistics_type>() != 0 {
        return None;
    }

    Some(unsafe { &mut *Pointer })
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
    File: Unique_file_identifier_type,
    Statistics: *mut Statistics_type,
) -> u32 {
    println!("Getting file statistics : {:?}", File);

    Into_u32(move || {
        println!("Getting file statistics : {:?}", File);

        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Statistics =
            Statistics_from_mutable_pointer(Statistics).ok_or(Error_type::Invalid_input)?;

        *Statistics = Get_file_system_instance()
            .Get_statistics(File, Task_identifier)
            .expect("Failed to get file statistics.");

        println!("Statistics : {:?}", *Statistics);

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
    File: Unique_file_identifier_type,
    Mode: *mut Mode_type,
) -> u32 {
    println!("Getting file access mode : {:?}", File);

    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        if Mode.is_null() {
            return Err(Error_type::Invalid_input);
        }

        Mode.write(Get_file_system_instance().Get_mode(File, Task_identifier)?);

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
pub extern "C" fn Xila_file_system_close(File: Unique_file_identifier_type) -> u32 {
    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

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
    File: Unique_file_identifier_type,
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
    File: Unique_file_identifier_type,
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
pub unsafe extern "C" fn Xila_file_system_is_terminal(
    File: Unique_file_identifier_type,
    Is_a_terminal: *mut bool,
) -> u32 {
    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        if Is_a_terminal.is_null() {
            return Err(Error_type::Invalid_input);
        }

        *Is_a_terminal = Get_file_system_instance().Is_a_terminal(File, Task_identifier)?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdin(File: Unique_file_identifier_type) -> bool {
    let (_, File) = File.Split();

    println!(
        "Checking if file is stdin : {:?} : {:?}",
        File,
        File == File_identifier_type::Standard_in
    );

    File == File_identifier_type::Standard_in
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stderr(File: Unique_file_identifier_type) -> bool {
    let (_, File) = File.Split();

    println!(
        "Checking if file is stderr : {:?} : {:?}",
        File,
        File == File_identifier_type::Standard_error
    );

    File == File_identifier_type::Standard_error
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdout(File: Unique_file_identifier_type) -> bool {
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
    Mode: Mode_type,
    Open: Open_type,
    Status: Status_type,
    File: *mut Unique_file_identifier_type,
) -> u32 {
    Into_u32(move || {
        let Path = std::ffi::CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_input)?;

        let Flags = Flags_type::New(Mode, Some(Open), Some(Status));

        println!("Opening file : {:?} with flags : {:?}", Path, Flags);

        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        *File = Get_file_system_instance()
            .Open(&Path, Flags, Task)
            .expect("Failed to open file");

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
            .map_err(|_| Error_type::Invalid_input)?;

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

/// This function is used to open a directory.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_open_directory(
    Path: *const c_char,
    Directory: *mut Unique_file_identifier_type,
) -> u32 {
    Into_u32(move || {
        let Path = CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_input)?;

        println!("Opening directory : {:?}", Path);

        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        *Directory = Get_file_system_instance()
            .Open_directory(&Path, Task)
            .expect("Failed to open directory");

        println!("Directory : {:?}", *Directory);

        Ok(())
    })
}

/// This function is used to read a directory.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_read_directory(
    File: Unique_file_identifier_type,
    Entry_name: *mut *mut i8,
    Entry_type: *mut Type_type,
    Entry_size: *mut Size_type,
    Entry_inode: *mut Inode_type,
) -> u32 {
    println!("Reading directory : {:?}", File);

    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Entry = Get_file_system_instance().Read_directory(File, Task)?;

        if let Some(Entry) = Entry {
            *Entry_name = CString::new(Entry.Get_name().as_str()).unwrap().into_raw();
            *Entry_type = Entry.Get_type();
            *Entry_size = Entry.Get_size();
            *Entry_inode = Entry.Get_inode();
        } else {
            *Entry_name = null_mut();
        }

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_close_directory(Directory: Unique_file_identifier_type) -> u32 {
    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        Get_file_system_instance().Close_directory(Directory, Task)?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_rewind_directory(Directory: Unique_file_identifier_type) -> u32 {
    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        Get_file_system_instance().Rewind_directory(Directory, Task)?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_flush(File: Unique_file_identifier_type, _: bool) -> u32 {
    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        Get_file_system_instance().Flush(File, Task)?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_read_link_at(
    _Directory: Unique_file_identifier_type,
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
    File: Unique_file_identifier_type,
    Offset: i64,
    Whence: Whence_type,
    Position: *mut Size_type,
) -> u32 {
    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Current_position = Position_type::From_whence(Whence, Offset);

        println!(
            "Setting position : {:?} : {:?} : {:?}",
            File, Current_position, Position
        );

        *Position = Get_file_system_instance().Set_position(File, &Current_position, Task)?;

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
            .map_err(|_| Error_type::Invalid_input)?;

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
            .map_err(|_| Error_type::Invalid_input)?;

        let New_path = CStr::from_ptr(New_path)
            .to_str()
            .map_err(|_| Error_type::Invalid_input)?;

        println!("Renaming : {:?} to : {:?}", Old_path, New_path);

        Get_file_system_instance().Rename(&Old_path, &New_path)?;

        Ok(())
    })
}
