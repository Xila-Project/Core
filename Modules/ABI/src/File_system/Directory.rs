extern crate alloc;

use alloc::ffi::CString;

use core::{
    ffi::{c_char, CStr},
    ptr::null_mut,
};

use File_system::Error_type;
use Task::Get_instance as Get_task_manager_instance;
use Virtual_file_system::Get_instance as Get_file_system_instance;

use super::{
    Into_u32, Xila_inode_type, Xila_size_type, Xila_type_type, Xila_unique_file_identifier_type,
};

/// This function is used to open a directory.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn Xila_file_system_open_directory(
    Path: *const c_char,
    Directory: *mut Xila_unique_file_identifier_type,
) -> u32 {
    Into_u32(move || {
        let Path = CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        println!("Opening directory : {:?}", Path);

        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        *Directory = Get_file_system_instance()
            .Open_directory(&Path, Task)
            .expect("Failed to open directory")
            .Into_inner();

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
    File: Xila_unique_file_identifier_type,
    Entry_name: *mut *mut i8,
    Entry_type: *mut Xila_type_type,
    Entry_size: *mut Xila_size_type,
    Entry_inode: *mut Xila_inode_type,
) -> u32 {
    println!("Reading directory : {:?}", File);

    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        let Entry = Get_file_system_instance().Read_directory(File, Task)?;

        if let Some(Entry) = Entry {
            *Entry_name = CString::new(Entry.Get_name().as_str()).unwrap().into_raw();
            *Entry_type = Entry.Get_type().into();
            *Entry_size = Entry.Get_size().As_u64();
            *Entry_inode = Entry.Get_inode().into();
        } else {
            *Entry_name = null_mut();
        }

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_close_directory(
    Directory: Xila_unique_file_identifier_type,
) -> u32 {
    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Directory = File_system::Unique_file_identifier_type::From_raw(Directory);

        Get_file_system_instance().Close_directory(Directory, Task)?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_rewind_directory(
    Directory: Xila_unique_file_identifier_type,
) -> u32 {
    Into_u32(move || {
        let Task = Get_task_manager_instance()
            .Get_current_task_identifier()
            .map_err(|_| Error_type::Failed_to_get_task_informations)?;

        let Directory = File_system::Unique_file_identifier_type::From_raw(Directory);

        Get_file_system_instance().Rewind_directory(Directory, Task)?;

        Ok(())
    })
}
