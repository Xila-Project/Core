use crate::{XilaFileIdentifier, XilaFileSystemMode, XilaFileSystemOpen, XilaFileSystemStatus};
use abi_context::{FileIdentifier, get_instance as get_context_instance};
use alloc::{borrow::ToOwned, ffi::CString};
use core::{
    ffi::{CStr, c_char},
    ptr::null_mut,
};
use file_system::{AccessFlags, CreateFlags, Error, Flags, Path, StateFlags};

use log::debug;
use virtual_file_system::{
    SynchronousDirectory, SynchronousFile, get_instance as get_file_system_instance,
};

use super::{XilaFileKind, XilaFileSystemInode, XilaFileSystemSize, into_u32};

/// This function is used to open a file or directory at a given directory.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_open_at(
    directory: XilaFileIdentifier,
    relative_path: *const c_char,
    is_directory: bool,
    mode: XilaFileSystemMode,
    open: XilaFileSystemOpen,
    status: XilaFileSystemStatus,
    out: *mut XilaFileIdentifier,
) -> u32 {
    into_u32(move || unsafe {
        if relative_path.is_null() || out.is_null() {
            Err(Error::InvalidParameter)?;
        }

        let relative_path = CStr::from_ptr(relative_path)
            .to_str()
            .map_err(|_| Error::InvalidParameter)?;

        let context = get_context_instance();

        let task = context.get_current_task_identifier();

        let virtual_file_system = get_file_system_instance();

        let (parent_identifier, full_path) = if relative_path == "/" {
            (None, Path::ROOT.to_owned())
        } else {
            let identifier = FileIdentifier::new(directory);
            let path = context
                .get_full_path(task, identifier, relative_path)
                .ok_or(Error::InvalidParameter)?;
            (Some(identifier), path)
        };

        if is_directory {
            let directory = SynchronousDirectory::open(virtual_file_system, task, relative_path)?;
            *out = context
                .insert_directory(task, parent_identifier, relative_path, directory)
                .ok_or(Error::InvalidIdentifier)?
                .into_inner();
        } else {
            let mode = AccessFlags::from_bits_truncate(mode);
            let open = CreateFlags::from_bits_truncate(open);
            let status = StateFlags::from_bits_truncate(status);

            let flags = Flags::new(mode, Some(open), Some(status));

            let file = SynchronousFile::open(virtual_file_system, task, &full_path, flags)?;

            *out = context
                .insert_file(task, file, None)
                .ok_or(Error::InvalidIdentifier)?
                .into_inner();
        }

        Ok(())
    })
}

/// This function is used to open a directory.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_open_directory(
    path: *const c_char,
    directory: *mut XilaFileIdentifier,
) -> u32 {
    unsafe {
        into_u32(move || {
            if path.is_null() || directory.is_null() {
                Err(Error::InvalidParameter)?;
            }

            let path = CStr::from_ptr(path)
                .to_str()
                .map_err(|_| Error::InvalidParameter)?;

            let context = get_context_instance();

            let task = context.get_current_task_identifier();

            debug!("Opening directory {path:?} for task {task:?}");

            let d = SynchronousDirectory::open(get_file_system_instance(), task, path)?;

            *directory = context
                .insert_directory(task, None, path, d)
                .ok_or(Error::InvalidIdentifier)?
                .into_inner();

            Ok(())
        })
    }
}

/// This function is used to read a directory.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_file_system_read_directory(
    directory: XilaFileIdentifier,
    entry_name: *mut *const c_char,
    entry_type: *mut XilaFileKind,
    entry_size: *mut XilaFileSystemSize,
    entry_inode: *mut XilaFileSystemInode,
) -> u32 {
    unsafe {
        into_u32(move || {
            let task = get_context_instance().get_current_task_identifier();

            debug!("Reading directory {directory:?} for task {task:?}");

            let entry = get_context_instance()
                .perform_operation_on_directory(directory.into(), SynchronousDirectory::read)
                .ok_or(Error::InvalidIdentifier)??;

            if let Some(entry) = entry {
                *entry_name = CString::new(entry.name.as_str()).unwrap().into_raw();
                *entry_type = entry.kind.into();
                *entry_size = entry.size;
                *entry_inode = entry.inode;
            } else {
                *entry_name = null_mut();
            }

            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_close_directory(directory: XilaFileIdentifier) -> u32 {
    into_u32(move || {
        log::information!("Closing directory {directory:?} ");

        let d = get_context_instance()
            .remove_directory(directory.into())
            .ok_or(Error::InvalidIdentifier)?;

        d.close(get_file_system_instance())?;

        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_rewind_directory(directory: XilaFileIdentifier) -> u32 {
    into_u32(move || {
        debug!("Rewinding directory {directory:?} ");

        get_context_instance()
            .perform_operation_on_directory(directory.into(), SynchronousDirectory::rewind)
            .ok_or(Error::InvalidIdentifier)??;

        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xila_file_system_directory_set_position(
    directory: XilaFileIdentifier,
    offset: XilaFileSystemSize,
) -> u32 {
    into_u32(move || {
        debug!("Setting position in directory {directory:?} to offset {offset}");

        get_context_instance()
            .perform_operation_on_directory(directory.into(), |d| d.set_position(offset))
            .ok_or(Error::InvalidIdentifier)??;

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use abi_context::{FileIdentifier, get_instance as get_context_instance};
    use alloc::{ffi::CString, format, vec::Vec};
    use file_system::{AccessFlags, CreateFlags, MemoryDevice, PathOwned};
    use task::{TaskIdentifier, test};
    use virtual_file_system::{Directory, File, VirtualFileSystem};

    async fn initialize() -> (TaskIdentifier, &'static VirtualFileSystem<'static>) {
        let users_manager = users::initialize();

        let time_manager = time::initialize(&drivers_std::devices::TimeDevice).unwrap();

        let task_manager = task::initialize();

        let task = task::get_instance().get_current_task_identifier().await;

        let device = MemoryDevice::<512>::new_static(1024 * 512);

        let cache_size = 256;

        little_fs::FileSystem::format(device, cache_size).unwrap();
        let file_system = little_fs::FileSystem::new(device, cache_size).unwrap();

        let virtual_file_system = virtual_file_system::initialize(
            task_manager,
            users_manager,
            time_manager,
            file_system,
            None,
        )
        .unwrap();

        (task, virtual_file_system)
    }

    #[test]
    async fn test_null_pointer_handling() {
        // Test that functions properly handle null pointers and return appropriate error codes
        let (_task, _vfs) = initialize().await;

        let context = get_context_instance();

        // Test open directory with null path
        let mut directory_id: XilaFileIdentifier = 0;
        let result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(core::ptr::null(), &mut directory_id)
            })
            .await;
        assert_ne!(result, 0, "Opening directory with null path should fail");

        // Test read directory with null output pointers
        let invalid_handle: XilaFileIdentifier = 9999;
        let result = context
            .call_abi(async || unsafe {
                xila_file_system_read_directory(
                    invalid_handle,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                )
            })
            .await;
        assert_ne!(
            result, 0,
            "Reading directory with null pointers should fail"
        );
    }

    #[test]
    async fn test_invalid_handle_operations() {
        initialize().await; // Ensure the test environment is initialized
        // Test operations on invalid directory handles
        let invalid_handle: XilaFileIdentifier = 9999;
        let context = get_context_instance();

        // Test close with invalid handle
        let result = context
            .call_abi(|| async { xila_file_system_close_directory(invalid_handle) })
            .await;
        assert_ne!(result, 0, "Closing invalid directory handle should fail");

        // Test rewind with invalid handle
        let result = context
            .call_abi(|| async { xila_file_system_rewind_directory(invalid_handle) })
            .await;
        assert_ne!(result, 0, "Rewinding invalid directory handle should fail");

        // Test set position with invalid handle
        let result = context
            .call_abi(|| async { xila_file_system_directory_set_position(invalid_handle, 0) })
            .await;
        assert_ne!(
            result, 0,
            "Setting position on invalid directory handle should fail"
        );
    }

    #[test]
    async fn test_read_directory_parameter_validation() {
        initialize().await; // Ensure the test environment is initialized

        // Test that read directory validates its parameters properly
        let invalid_handle: XilaFileIdentifier = 0;
        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: XilaFileKind = XilaFileKind::File;
        let mut entry_size: XilaFileSystemSize = 0;
        let mut entry_inode: XilaFileSystemInode = 0;
        let context = get_context_instance();

        // Test with invalid handle but valid output pointers
        let result = context
            .call_abi(async || unsafe {
                xila_file_system_read_directory(
                    invalid_handle,
                    &mut entry_name,
                    &mut entry_type,
                    &mut entry_size,
                    &mut entry_inode,
                )
            })
            .await;
        assert_ne!(result, 0, "Reading from invalid handle should fail");
    }

    #[test]
    async fn test_set_position_boundary_values() {
        initialize().await; // Ensure the test environment is initialized
        // Test set position with boundary values
        let invalid_handle: XilaFileIdentifier = 9999;
        let context = get_context_instance();

        // Test with maximum value
        let result = context
            .call_abi(|| async {
                xila_file_system_directory_set_position(invalid_handle, u64::MAX)
            })
            .await;
        assert_ne!(
            result, 0,
            "Setting position with max value on invalid handle should fail"
        );

        // Test with zero value
        let result = context
            .call_abi(|| async { xila_file_system_directory_set_position(invalid_handle, 0) })
            .await;
        assert_ne!(
            result, 0,
            "Setting position with zero on invalid handle should fail"
        );
    }

    #[test]
    async fn test_open_directory_valid_path() {
        let (_task, _vfs) = initialize().await;
        let context = get_context_instance();

        let path = CString::new("/").unwrap();
        let mut directory_id: XilaFileIdentifier = 0;

        let result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(result, 0, "Opening root directory should succeed");
        assert_ne!(directory_id, 0, "Directory identifier should be non-zero");

        // Clean up
        let close_result = context
            .call_abi(|| async { xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[test]
    async fn test_open_directory_invalid_path() {
        let (_task, _vfs) = initialize().await;
        let context = get_context_instance();

        let path = CString::new("/nonexistent").unwrap();
        let mut directory_id: XilaFileIdentifier = 0;

        let result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;

        assert_ne!(result, 0, "Opening nonexistent directory should fail");
    }

    #[test]
    async fn test_open_directory_null_path() {
        let (_task, _vfs) = initialize().await;
        let context = get_context_instance();

        let mut directory_id: XilaFileIdentifier = 0;

        let result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(core::ptr::null(), &mut directory_id)
            })
            .await;

        assert_ne!(result, 0, "Opening directory with null path should fail");
    }

    #[test]
    async fn test_read_directory_entries() {
        let (_task, vfs) = initialize().await;
        let task = _task;
        let context = get_context_instance();

        // Create some test files and directories
        Directory::create(vfs, task, "/test_read_directory_entries")
            .await
            .unwrap();

        let test_file = File::open(
            vfs,
            task,
            "/test_read_directory_entries.txt",
            Flags::new(AccessFlags::Write, Some(CreateFlags::Create), None),
        )
        .await
        .unwrap();

        test_file.close(vfs).await.unwrap();

        // Open root directory
        let path = CString::new("/").unwrap();
        let mut directory_id: XilaFileIdentifier = 0;

        let open_result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening root directory should succeed");

        // Read directory entries

        let mut entries_found = Vec::new();

        // Read all entries
        loop {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: XilaFileKind = XilaFileKind::File;
            let mut entry_size: XilaFileSystemSize = 0;
            let mut entry_inode: XilaFileSystemInode = 0;

            let read_result = context
                .call_abi(async || unsafe {
                    xila_file_system_read_directory(
                        directory_id,
                        &mut entry_name,
                        &mut entry_type,
                        &mut entry_size,
                        &mut entry_inode,
                    )
                })
                .await;

            assert_eq!(read_result, 0, "Reading directory entry should succeed");

            if entry_name.is_null() {
                break; // End of directory
            }

            let name = unsafe { CStr::from_ptr(entry_name).to_string_lossy().into_owned() };
            entries_found.push((name, entry_type));

            // Free the allocated string
            unsafe {
                let _ = CString::from_raw(entry_name as *mut c_char);
            }
        }

        // Verify we found the expected entries
        assert!(
            entries_found.len() >= 2,
            "Should find at least current and parent directories"
        );

        // Check for current and parent directory entries
        let has_current = entries_found.iter().any(|(name, _)| name == ".");
        let has_parent = entries_found.iter().any(|(name, _)| name == "..");

        assert!(has_current, "Should find current directory entry");
        assert!(has_parent, "Should find parent directory entry");

        // Clean up
        let close_result = context
            .call_abi(|| async { xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[test]
    async fn test_read_directory_invalid_handle() {
        let (_task, _vfs) = initialize().await;
        let context = get_context_instance();

        let invalid_directory_id: XilaFileIdentifier = 9999;
        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: XilaFileKind = XilaFileKind::File;
        let mut entry_size: XilaFileSystemSize = 0;
        let mut entry_inode: XilaFileSystemInode = 0;

        let result = context
            .call_abi(async || unsafe {
                xila_file_system_read_directory(
                    invalid_directory_id,
                    &mut entry_name,
                    &mut entry_type,
                    &mut entry_size,
                    &mut entry_inode,
                )
            })
            .await;

        assert_ne!(
            result, 0,
            "Reading from invalid directory handle should fail"
        );
    }

    #[test]
    async fn test_close_directory_valid_handle() {
        let (_task, _vfs) = initialize().await;
        let context = get_context_instance();

        let path = CString::new("/").unwrap();
        let mut directory_id: XilaFileIdentifier = 0;

        // Open directory
        let open_result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening directory should succeed");

        // Close directory
        let close_result = context
            .call_abi(|| async { xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[test]
    async fn test_close_directory_invalid_handle() {
        let (_task, _vfs) = initialize().await;
        let context = get_context_instance();

        let invalid_directory_id: XilaFileIdentifier = 9999;

        let result = context
            .call_abi(|| async { xila_file_system_close_directory(invalid_directory_id) })
            .await;
        assert_ne!(result, 0, "Closing invalid directory handle should fail");
    }

    #[test]
    async fn test_rewind_directory() {
        let (_task, vfs) = initialize().await;
        let task = _task;
        let context = get_context_instance();

        // Create some test files
        for i in 0..3 {
            let path = format!("/test_rewind_directory_{i}.txt");

            let test_file = vfs
                .open(
                    &PathOwned::new(path).unwrap(),
                    file_system::Flags::new(AccessFlags::Write, Some(CreateFlags::Create), None),
                    task,
                )
                .await
                .unwrap();
            test_file.close(vfs).await.unwrap();
        }

        // Open directory
        let path = CString::new("/").unwrap();
        let mut directory_id: XilaFileIdentifier = 0;

        let open_result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening directory should succeed");

        // Read a few entries
        for _ in 0..2 {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: XilaFileKind = XilaFileKind::File;
            let mut entry_size: XilaFileSystemSize = 0;
            let mut entry_inode: XilaFileSystemInode = 0;

            let read_result = context
                .call_abi(async || unsafe {
                    xila_file_system_read_directory(
                        directory_id,
                        &mut entry_name,
                        &mut entry_type,
                        &mut entry_size,
                        &mut entry_inode,
                    )
                })
                .await;
            assert_eq!(read_result, 0, "Reading directory should succeed");

            if !entry_name.is_null() {
                unsafe {
                    let _ = CString::from_raw(entry_name as *mut c_char);
                }
            }
        }

        // Rewind directory
        let rewind_result = context
            .call_abi(|| async { xila_file_system_rewind_directory(directory_id) })
            .await;
        assert_eq!(rewind_result, 0, "Rewinding directory should succeed");

        // Read first entry again - should be "."
        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: XilaFileKind = XilaFileKind::File;
        let mut entry_size: XilaFileSystemSize = 0;
        let mut entry_inode: XilaFileSystemInode = 0;

        let read_result = context
            .call_abi(async || unsafe {
                xila_file_system_read_directory(
                    directory_id,
                    &mut entry_name,
                    &mut entry_type,
                    &mut entry_size,
                    &mut entry_inode,
                )
            })
            .await;
        assert_eq!(
            read_result, 0,
            "Reading directory after rewind should succeed"
        );
        assert!(!entry_name.is_null(), "Entry name should not be null");

        let name = unsafe { CStr::from_ptr(entry_name).to_string_lossy() };
        assert_eq!(
            name, ".",
            "First entry after rewind should be current directory"
        );

        unsafe {
            let _ = CString::from_raw(entry_name as *mut c_char);
        }

        // Clean up
        let close_result = context
            .call_abi(|| async { xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[test]
    async fn test_rewind_directory_invalid_handle() {
        let (_task, _vfs) = initialize().await;
        let context = get_context_instance();

        let invalid_directory_id: XilaFileIdentifier = 9999;

        let result = context
            .call_abi(|| async { xila_file_system_rewind_directory(invalid_directory_id) })
            .await;
        assert_ne!(result, 0, "Rewinding invalid directory handle should fail");
    }

    #[test]
    async fn test_directory_set_position() {
        let (_task, vfs) = initialize().await;
        let task = _task;
        let context = get_context_instance();

        // Create some test files
        for i in 0..5 {
            let path = format!("/test_file_{i}.txt");

            let test_file = vfs
                .open(
                    &PathOwned::new(path).unwrap(),
                    file_system::Flags::new(AccessFlags::Write, Some(CreateFlags::Create), None),
                    task,
                )
                .await
                .unwrap();
            test_file.close(vfs).await.unwrap();
        }

        // Open directory
        let path = CString::new("/").unwrap();
        let mut directory_id: XilaFileIdentifier = 0;

        let open_result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening directory should succeed");

        // Read a few entries to advance position
        for _ in 0..3 {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: XilaFileKind = XilaFileKind::File;
            let mut entry_size: XilaFileSystemSize = 0;
            let mut entry_inode: XilaFileSystemInode = 0;

            let read_result = context
                .call_abi(async || unsafe {
                    xila_file_system_read_directory(
                        directory_id,
                        &mut entry_name,
                        &mut entry_type,
                        &mut entry_size,
                        &mut entry_inode,
                    )
                })
                .await;
            assert_eq!(read_result, 0, "Reading directory should succeed");

            if !entry_name.is_null() {
                unsafe {
                    let _ = CString::from_raw(entry_name as *mut c_char);
                }
            }
        }

        // Set position back to start
        let set_position_result = context
            .call_abi(|| async { xila_file_system_directory_set_position(directory_id, 0) })
            .await;
        assert_eq!(
            set_position_result, 0,
            "Setting directory position should succeed"
        );

        // Read first entry - should be "." again
        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: XilaFileKind = XilaFileKind::File;
        let mut entry_size: XilaFileSystemSize = 0;
        let mut entry_inode: XilaFileSystemInode = 0;

        let read_result = context
            .call_abi(async || unsafe {
                xila_file_system_read_directory(
                    directory_id,
                    &mut entry_name,
                    &mut entry_type,
                    &mut entry_size,
                    &mut entry_inode,
                )
            })
            .await;
        assert_eq!(
            read_result, 0,
            "Reading directory after position reset should succeed"
        );
        assert!(!entry_name.is_null(), "Entry name should not be null");

        let name = unsafe { CStr::from_ptr(entry_name).to_string_lossy() };
        assert_eq!(
            name, ".",
            "First entry after position reset should be current directory"
        );

        unsafe {
            let _ = CString::from_raw(entry_name as *mut c_char);
        }

        // Clean up
        let close_result = context
            .call_abi(|| async { xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[test]
    async fn test_directory_set_position_invalid_handle() {
        let (_task, _vfs) = initialize().await;
        let context = get_context_instance();

        let invalid_directory_id: XilaFileIdentifier = 9999;

        let result = context
            .call_abi(|| async { xila_file_system_directory_set_position(invalid_directory_id, 0) })
            .await;
        assert_ne!(
            result, 0,
            "Setting position on invalid directory handle should fail"
        );
    }

    #[test]
    async fn test_directory_operations_sequence() {
        let (_task, vfs) = initialize().await;
        let task = _task;
        let context = get_context_instance();

        // Create test structure
        Directory::create(vfs, task, "/test_dir").await.unwrap();
        Directory::create(vfs, task, "/test_dir/subdir")
            .await
            .unwrap();

        let test_file = File::open(
            vfs,
            task,
            &"/test_dir/file.txt",
            file_system::Flags::new(AccessFlags::Write, Some(CreateFlags::Create), None),
        )
        .await
        .unwrap();
        test_file.close(vfs).await.unwrap();

        // Test opening the created directory
        let path = CString::new("/test_dir").unwrap();
        let mut directory_id: XilaFileIdentifier = 0;

        let open_result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening test directory should succeed");

        // Count entries
        let mut entry_count = 0;
        loop {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: XilaFileKind = XilaFileKind::File;
            let mut entry_size: XilaFileSystemSize = 0;
            let mut entry_inode: XilaFileSystemInode = 0;

            let read_result = context
                .call_abi(async || unsafe {
                    xila_file_system_read_directory(
                        directory_id,
                        &mut entry_name,
                        &mut entry_type,
                        &mut entry_size,
                        &mut entry_inode,
                    )
                })
                .await;
            assert_eq!(read_result, 0, "Reading directory should succeed");

            if entry_name.is_null() {
                break;
            }

            entry_count += 1;
            unsafe {
                let _ = CString::from_raw(entry_name as *mut c_char);
            }
        }

        // Should have at least: ., .., subdir, file.txt
        assert!(
            entry_count >= 4,
            "Should find at least 4 entries in test directory"
        );

        // Test rewind and count again
        let rewind_result = context
            .call_abi(|| async { xila_file_system_rewind_directory(directory_id) })
            .await;
        assert_eq!(rewind_result, 0, "Rewinding directory should succeed");

        let mut rewind_count = 0;
        loop {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: XilaFileKind = XilaFileKind::File;
            let mut entry_size: XilaFileSystemSize = 0;
            let mut entry_inode: XilaFileSystemInode = 0;

            let read_result = context
                .call_abi(async || unsafe {
                    xila_file_system_read_directory(
                        directory_id,
                        &mut entry_name,
                        &mut entry_type,
                        &mut entry_size,
                        &mut entry_inode,
                    )
                })
                .await;
            assert_eq!(
                read_result, 0,
                "Reading directory after rewind should succeed"
            );

            if entry_name.is_null() {
                break;
            }

            rewind_count += 1;
            unsafe {
                let _ = CString::from_raw(entry_name as *mut c_char);
            }
        }

        assert_eq!(
            entry_count, rewind_count,
            "Entry count should be same after rewind"
        );

        // Clean up
        let close_result = context
            .call_abi(|| async { xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[test]
    async fn test_directory_operations_error_handling() {
        initialize().await;

        let context = get_context_instance();

        // Test null pointer handling
        let mut directory_id: XilaFileIdentifier = 0;

        // Test with null path
        let result = context
            .call_abi(async || unsafe {
                xila_file_system_open_directory(core::ptr::null(), &mut directory_id)
            })
            .await;
        assert_ne!(result, 0, "Null path should cause error");

        // Test invalid operations on invalid handles
        let invalid_handle = FileIdentifier::INVALID.into();

        let close_result = context
            .call_abi(|| async { xila_file_system_close_directory(invalid_handle) })
            .await;
        assert_ne!(close_result, 0, "Invalid close should fail");

        let rewind_result = context
            .call_abi(|| async { xila_file_system_rewind_directory(invalid_handle) })
            .await;
        assert_ne!(rewind_result, 0, "Invalid rewind should fail");

        let set_pos_result = context
            .call_abi(|| async { xila_file_system_directory_set_position(invalid_handle, 0) })
            .await;
        assert_ne!(set_pos_result, 0, "Invalid set position should fail");

        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: XilaFileKind = XilaFileKind::File;
        let mut entry_size: XilaFileSystemSize = 0;
        let mut entry_inode: XilaFileSystemInode = 0;

        let read_result = context
            .call_abi(async || unsafe {
                xila_file_system_read_directory(
                    invalid_handle,
                    &mut entry_name,
                    &mut entry_type,
                    &mut entry_size,
                    &mut entry_inode,
                )
            })
            .await;
        assert_ne!(read_result, 0, "Invalid read should fail");
    }
}
