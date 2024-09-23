/// This module implements the POSIX like file system C ABI.
use std::num::NonZeroU32;

use File_system::{
    Error_type, File_identifier_type, Get_instance as Get_file_system_instance, Mode_type,
    Result_type, Statistics_type, Unique_file_identifier_type,
};

use Task::Get_instance as Get_task_manager_instance;

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
pub unsafe extern "C" fn Xila_get_file_statistics(
    File: Unique_file_identifier_type,
    Statistics: *mut Statistics_type,
) -> u32 {
    Into_u32(move || {
        let Task_identifier = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Statistics =
            Statistics_from_mutable_pointer(Statistics).ok_or(Error_type::Invalid_input)?;

        *Statistics = Get_file_system_instance()
            .Get_statistics(File, Task_identifier)
            .expect("Failed to get file statistics.");

        Ok(())
    })
}

#[no_mangle]
pub static Xila_file_system_mode_read_bit: u8 = Mode_type::Read_bit;
#[no_mangle]
pub static Xila_file_system_mode_write_bit: u8 = Mode_type::Write_bit;

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
    println!("Checking if file is a terminal : {:?}", File);

    Is_a_terminal.write(false);

    0
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdin(File: Unique_file_identifier_type) -> bool {
    let (_, File) = File.Split();

    println!(
        "Checking if file is stdin : {:?} : {:?}",
        File,
        File == File_identifier_type::Stdin
    );

    File == File_identifier_type::Stdin
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stderr(File: Unique_file_identifier_type) -> bool {
    let (_, File) = File.Split();

    println!(
        "Checking if file is stderr : {:?} : {:?}",
        File,
        File == File_identifier_type::Stderr
    );

    File == File_identifier_type::Stderr
}

#[no_mangle]
pub extern "C" fn Xila_file_system_is_stdout(File: Unique_file_identifier_type) -> bool {
    let (_, File) = File.Split();

    println!(
        "Checking if file is stdout : {:?} : {:?}",
        File,
        File == File_identifier_type::Stdout
    );

    File == File_identifier_type::Stdout
}
