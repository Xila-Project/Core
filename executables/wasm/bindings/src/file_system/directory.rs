use core::ffi::{CStr, c_char};

use crate::file_system::{FileSystemItem, XilaFileSystemItem, into_result};
use xila::abi_declarations::{
    XilaFileKind, XilaFileSystemAccess, XilaFileSystemInode, XilaFileSystemResult,
    XilaFileSystemSize, XilaFileSystemStatistics, xila_file_system_directory_close,
    xila_file_system_directory_get_access, xila_file_system_directory_get_statistics,
    xila_file_system_directory_read, xila_file_system_directory_rewind,
    xila_file_system_directory_set_position,
};
use xila::{log, virtual_file_system};

macro_rules! with_directory {
    ($ptr:expr, $dir_var:ident => $body:expr) => {
        unsafe {
            match FileSystemItem::borrow_from_raw($ptr as _) {
                FileSystemItem::Directory($dir_var) => $body,
                _ => virtual_file_system::Error::InvalidParameter.into(),
            }
        }
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_directory_close(
    item: *mut XilaFileSystemItem,
) -> XilaFileSystemResult {
    log::information!("Closing directory {:?}", item);
    unsafe {
        let directory = FileSystemItem::own_from_raw(item as _);

        match *directory {
            FileSystemItem::Directory(mut directory) => {
                xila_file_system_directory_close(&mut directory.directory as _)
            }
            _ => virtual_file_system::Error::InvalidParameter.into(),
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_directory_rewind(
    item: *mut XilaFileSystemItem,
) -> XilaFileSystemResult {
    log::information!("Rewinding directory {:?}", item);
    with_directory!(item, d => xila_file_system_directory_rewind(&mut d.directory as _))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_directory_read(
    item: *mut XilaFileSystemItem,
    entry_name: *mut *const c_char,
    entry_type: *mut XilaFileKind,
    entry_size: *mut XilaFileSystemSize,
    entry_inode: *mut XilaFileSystemInode,
) -> XilaFileSystemResult {
    log::information!("Reading directory {:?} ", item);
    with_directory!(item, dir => xila_file_system_directory_read(
        &mut dir.directory as _,
        entry_name,
        entry_type,
        entry_size,
        entry_inode,
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_directory_set_position(
    item: *mut XilaFileSystemItem,
    offset: XilaFileSystemSize,
) -> XilaFileSystemResult {
    log::information!(
        "Setting position in directory {:?} to offset {}",
        item,
        offset
    );
    with_directory!(item, dir => xila_file_system_directory_set_position(&mut dir.directory as _, offset))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wasm_file_system_directory_get_statistics(
    item: *mut XilaFileSystemItem,
    statistics: *mut XilaFileSystemStatistics,
) -> XilaFileSystemResult {
    log::information!("Getting statistics for directory {:?} ", item);
    with_directory!(item, dir => xila_file_system_directory_get_statistics(&mut dir.directory as _, statistics))
}

#[unsafe(no_mangle)]
pub extern "C" fn __wasm_file_system_directory_get_access(
    item: *mut XilaFileSystemItem,
    access: *mut XilaFileSystemAccess,
) -> XilaFileSystemResult {
    log::information!("Getting access for directory {:?} ", item);
    with_directory!(item, dir => xila_file_system_directory_get_access(&mut dir.directory as _, access))
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_directory_get_state(
    item: *mut XilaFileSystemItem,
    state: *mut XilaFileSystemAccess,
) -> XilaFileSystemResult {
    log::information!("Getting state for directory {:?} ", item);
    with_directory!(item, dir => xila_file_system_directory_get_access(&mut dir.directory as _, state))
}
