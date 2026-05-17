use crate::{
    XilaFileSystemAccess, XilaFileSystemStatistics, XilaTaskIdentifier, abi_unsafe_function,
    parse_c_str,
};
use alloc::ffi::CString;
use core::{ffi::c_char, ptr::null_mut};
use log::debug;
use shared::generate_shadow_type;
use task::{TaskIdentifier, block_on};
use virtual_file_system::{SynchronousDirectory, get_instance as get_file_system_instance};

use super::{XilaFileKind, XilaFileSystemInode, XilaFileSystemSize};

generate_shadow_type!(XilaFileSystemDirectory, SynchronousDirectory, 40, 8);

abi_unsafe_function! {
    /// This function is used to open a directory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    fn xila_file_system_directory_open(
        task: XilaTaskIdentifier,
        path: *const c_char,
        out_directory: *mut XilaFileSystemDirectory,
    ) -> XilaFileSystemResult {
        let path = parse_c_str(path)?;
        let task = TaskIdentifier::from(task);

        debug!("Opening directory {path:?} for task {task:?}");

        let synchronous_directory =
            SynchronousDirectory::open(get_file_system_instance(), task, path)?;

        *out_directory = XilaFileSystemDirectory::from_real(synchronous_directory);

        Ok(())
    }
}

abi_unsafe_function! {
    /// This function is used to read a directory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    fn xila_file_system_directory_read(
        directory: *mut XilaFileSystemDirectory,
        entry_name: *mut *const c_char,
        entry_type: *mut XilaFileKind,
        entry_size: *mut XilaFileSystemSize,
        entry_inode: *mut XilaFileSystemInode,
    ) -> XilaFileSystemResult {
        debug!("Reading directory {directory:?}");

        let entry = (*directory).read()?;

        if let Some(entry) = entry {
            *entry_name = CString::new(entry.name.as_str()).unwrap().into_raw();
            *entry_type = entry.kind.into();
            *entry_size = entry.size;
            *entry_inode = entry.inode;
        } else {
            *entry_name = null_mut();
        }

        Ok(())
    }
}

abi_unsafe_function! {
    fn xila_file_system_directory_close(
        directory: *mut XilaFileSystemDirectory,
    ) -> XilaFileSystemResult {
        (*directory).close_internal(get_file_system_instance())
    }
}

abi_unsafe_function! {
    fn xila_file_system_directory_rewind(
        directory: *mut XilaFileSystemDirectory,
    ) -> XilaFileSystemResult {
        debug!("Rewinding directory {directory:?} ");

        (*directory).rewind()
    }
}

abi_unsafe_function! {
    fn xila_file_system_directory_set_position(
        directory: *mut XilaFileSystemDirectory,
        offset: XilaFileSystemSize,
    ) -> XilaFileSystemResult {
        debug!("Setting position in directory {directory:?} to offset {offset}");

        (*directory).set_position(offset)?;

        Ok(())
    }
}

abi_unsafe_function! {
    fn xila_file_system_directory_get_statistics(
        directory: *mut XilaFileSystemDirectory,
        statistics: *mut XilaFileSystemStatistics,
    ) -> XilaFileSystemResult {
        debug!("Getting statistics for directory {directory:?}");

        let result = (*directory).get_statistics()?;

        *statistics = XilaFileSystemStatistics::from_statistics(result);

        Ok(())
    }
}

abi_unsafe_function! {
    fn xila_file_system_directory_get_access(
        directory: *mut XilaFileSystemDirectory,
        access: *mut XilaFileSystemAccess,
    ) -> XilaFileSystemResult {
        debug!("Getting access flags for directory {directory:?}");

        let result = (*directory).get_access()?;

        access.write(result.bits());

        Ok(())
    }
}

abi_unsafe_function! {
    fn xila_file_system_directory_get_state(
        directory: *mut XilaFileSystemDirectory,
        state: *mut u8,
    ) -> XilaFileSystemResult {
        debug!("Getting state flags for directory {directory:?}");

        let result = (*directory).get_state()?;

        state.write(result.bits());

        Ok(())
    }
}

abi_unsafe_function! {
    /// This function is used to get the position in a file.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    fn xila_file_system_directory_create(
        task: XilaTaskIdentifier,
        path: *const c_char,
    ) -> XilaFileSystemResult {
        let path = parse_c_str(path)?;

        block_on(get_file_system_instance().create_directory(task.into(), &path))?;

        Ok(())
    }
}
