use alloc::ffi::CString;
use Log::Debug;

use core::{
    ffi::{c_char, CStr},
    ptr::null_mut,
};

use File_system::Error_type;
use Futures::block_on;
use Virtual_file_system::Get_instance as Get_file_system_instance;

use crate::Context::Get_instance as Get_context_instance;

use super::{
    Into_u32, Xila_file_system_inode_type, Xila_file_system_size_type, Xila_file_type_type,
    Xila_unique_file_identifier_type,
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
        if Path.is_null() || Directory.is_null() {
            Err(Error_type::Invalid_parameter)?;
        }

        let Path = CStr::from_ptr(Path)
            .to_str()
            .map_err(|_| Error_type::Invalid_parameter)?;

        let Task = Get_context_instance().Get_current_task_identifier();

        Debug!("Opening directory {Path:?} for task {Task:?}");

        *Directory = block_on(Get_file_system_instance().Open_directory(&Path, Task))?.Into_inner();

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
    Entry_name: *mut *const c_char,
    Entry_type: *mut Xila_file_type_type,
    Entry_size: *mut Xila_file_system_size_type,
    Entry_inode: *mut Xila_file_system_inode_type,
) -> u32 {
    Into_u32(move || {
        let Task = Get_context_instance().Get_current_task_identifier();

        Debug!("Reading directory {File:?} for task {Task:?}");

        let File = File_system::Unique_file_identifier_type::From_raw(File);

        let Entry = block_on(Get_file_system_instance().Read_directory(File, Task))?;

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
        let Task = Get_context_instance().Get_current_task_identifier();

        let Directory = File_system::Unique_file_identifier_type::From_raw(Directory);

        Debug!("Closing directory {Directory:?} for task {Task:?}");

        block_on(Get_file_system_instance().Close_directory(Directory, Task))?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_rewind_directory(
    Directory: Xila_unique_file_identifier_type,
) -> u32 {
    Into_u32(move || {
        let Task = Get_context_instance().Get_current_task_identifier();

        let Directory = File_system::Unique_file_identifier_type::From_raw(Directory);

        Debug!("Rewinding directory {Directory:?} for task {Task:?}");

        block_on(Get_file_system_instance().Rewind_directory(Directory, Task))?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn Xila_file_system_directory_set_position(
    Directory: Xila_unique_file_identifier_type,
    Offset: Xila_file_system_size_type,
) -> u32 {
    Into_u32(move || {
        let Task = Get_context_instance().Get_current_task_identifier();

        let Directory = File_system::Unique_file_identifier_type::From_raw(Directory);

        Debug!("Setting position in directory {Directory:?} to offset {Offset} for task {Task:?}");

        block_on(Get_file_system_instance().Set_position_directory(
            Directory,
            Offset.into(),
            Task,
        ))?;

        Ok(())
    })
}

#[cfg(test)]
mod Tests {

    use super::*;
    use crate::Context::Get_instance as Get_context_instance;
    use alloc::{ffi::CString, format, vec::Vec};
    use File_system::{
        Create_device, Create_file_system, Memory_device_type, Mode_type, Open_type,
        Path_owned_type,
    };
    use Task::{Task_identifier_type, Test};
    use Virtual_file_system::Virtual_file_system_type;

    async fn Initialize_test_environment() -> (
        Task_identifier_type,
        &'static Virtual_file_system_type<'static>,
    ) {
        let _ = Users::Initialize();

        let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

        let Cache_size = 256;

        LittleFS::File_system_type::Format(Device.clone(), Cache_size).unwrap();
        let File_system = LittleFS::File_system_type::New(Device, Cache_size).unwrap();

        let Virtual_file_system =
            Virtual_file_system::Initialize(Create_file_system!(File_system), None).unwrap();

        (Task, Virtual_file_system)
    }

    #[Test]
    async fn test_null_pointer_handling() {
        // Test that functions properly handle null pointers and return appropriate error codes
        let (_Task, _VFS) = Initialize_test_environment().await;

        let Context = Get_context_instance();

        // Test open directory with null path
        let mut directory_id: Xila_unique_file_identifier_type = 0;
        let result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(core::ptr::null(), &mut directory_id)
            })
            .await;
        assert_ne!(result, 0, "Opening directory with null path should fail");

        // Test read directory with null output pointers
        let invalid_handle: Xila_unique_file_identifier_type = 999999;
        let result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_read_directory(
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

    #[Test]
    async fn test_invalid_handle_operations() {
        Initialize_test_environment().await; // Ensure the test environment is initialized
                                             // Test operations on invalid directory handles
        let invalid_handle: Xila_unique_file_identifier_type = 999999;
        let Context = Get_context_instance();

        // Test close with invalid handle
        let result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(invalid_handle) })
            .await;
        assert_ne!(result, 0, "Closing invalid directory handle should fail");

        // Test rewind with invalid handle
        let result = Context
            .Call_ABI(|| async { Xila_file_system_rewind_directory(invalid_handle) })
            .await;
        assert_ne!(result, 0, "Rewinding invalid directory handle should fail");

        // Test set position with invalid handle
        let result = Context
            .Call_ABI(|| async { Xila_file_system_directory_set_position(invalid_handle, 0) })
            .await;
        assert_ne!(
            result, 0,
            "Setting position on invalid directory handle should fail"
        );
    }

    #[Test]
    async fn test_read_directory_parameter_validation() {
        Initialize_test_environment().await; // Ensure the test environment is initialized

        // Test that read directory validates its parameters properly
        let invalid_handle: Xila_unique_file_identifier_type = 0;
        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
        let mut entry_size: Xila_file_system_size_type = 0;
        let mut entry_inode: Xila_file_system_inode_type = 0;
        let Context = Get_context_instance();

        // Test with invalid handle but valid output pointers
        let result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_read_directory(
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

    #[Test]
    async fn test_set_position_boundary_values() {
        Initialize_test_environment().await; // Ensure the test environment is initialized
                                             // Test set position with boundary values
        let invalid_handle: Xila_unique_file_identifier_type = 999999;
        let Context = Get_context_instance();

        // Test with maximum value
        let result = Context
            .Call_ABI(|| async {
                Xila_file_system_directory_set_position(invalid_handle, u64::MAX)
            })
            .await;
        assert_ne!(
            result, 0,
            "Setting position with max value on invalid handle should fail"
        );

        // Test with zero value
        let result = Context
            .Call_ABI(|| async { Xila_file_system_directory_set_position(invalid_handle, 0) })
            .await;
        assert_ne!(
            result, 0,
            "Setting position with zero on invalid handle should fail"
        );
    }

    #[Test]
    async fn Test_open_directory_valid_path() {
        let (_Task, _VFS) = Initialize_test_environment().await;
        let Context = Get_context_instance();

        let path = CString::new("/").unwrap();
        let mut directory_id: Xila_unique_file_identifier_type = 0;

        let result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(result, 0, "Opening root directory should succeed");
        assert_ne!(directory_id, 0, "Directory identifier should be non-zero");

        // Clean up
        let close_result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[Test]
    async fn Test_open_directory_invalid_path() {
        let (_Task, _VFS) = Initialize_test_environment().await;
        let Context = Get_context_instance();

        let path = CString::new("/nonexistent").unwrap();
        let mut directory_id: Xila_unique_file_identifier_type = 0;

        let result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;

        assert_ne!(result, 0, "Opening nonexistent directory should fail");
    }

    #[Test]
    async fn Test_open_directory_null_path() {
        let (_Task, _VFS) = Initialize_test_environment().await;
        let Context = Get_context_instance();

        let mut directory_id: Xila_unique_file_identifier_type = 0;

        let result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(core::ptr::null(), &mut directory_id)
            })
            .await;

        assert_ne!(result, 0, "Opening directory with null path should fail");
    }

    #[Test]
    async fn Test_read_directory_entries() {
        let (_Task, VFS) = Initialize_test_environment().await;
        let Task = _Task;
        let Context = Get_context_instance();

        // Create some test files and directories
        VFS.Create_directory(&"/test_read_directory_entries", Task)
            .await
            .unwrap();

        let test_file = VFS
            .Open(
                &"/test_read_directory_entries.txt",
                File_system::Flags_type::New(
                    Mode_type::WRITE_ONLY,
                    Some(Open_type::CREATE_ONLY),
                    None,
                ),
                Task,
            )
            .await
            .unwrap();
        VFS.Close(test_file, Task).await.unwrap();

        // Open root directory
        let path = CString::new("/").unwrap();
        let mut directory_id: Xila_unique_file_identifier_type = 0;

        let open_result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening root directory should succeed");

        // Read directory entries

        let mut entries_found = Vec::new();

        // Read all entries
        loop {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
            let mut entry_size: Xila_file_system_size_type = 0;
            let mut entry_inode: Xila_file_system_inode_type = 0;

            let read_result = Context
                .Call_ABI(async || unsafe {
                    Xila_file_system_read_directory(
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
        let close_result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[Test]
    async fn Test_read_directory_invalid_handle() {
        let (_Task, _VFS) = Initialize_test_environment().await;
        let Context = Get_context_instance();

        let invalid_directory_id: Xila_unique_file_identifier_type = 999999;
        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
        let mut entry_size: Xila_file_system_size_type = 0;
        let mut entry_inode: Xila_file_system_inode_type = 0;

        let result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_read_directory(
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

    #[Test]
    async fn Test_close_directory_valid_handle() {
        let (_Task, _VFS) = Initialize_test_environment().await;
        let Context = Get_context_instance();

        let path = CString::new("/").unwrap();
        let mut directory_id: Xila_unique_file_identifier_type = 0;

        // Open directory
        let open_result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening directory should succeed");

        // Close directory
        let close_result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[Test]
    async fn Test_close_directory_invalid_handle() {
        let (_Task, _VFS) = Initialize_test_environment().await;
        let Context = Get_context_instance();

        let invalid_directory_id: Xila_unique_file_identifier_type = 999999;

        let result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(invalid_directory_id) })
            .await;
        assert_ne!(result, 0, "Closing invalid directory handle should fail");
    }

    #[Test]
    async fn Test_rewind_directory() {
        let (_Task, VFS) = Initialize_test_environment().await;
        let Task = _Task;
        let Context = Get_context_instance();

        // Create some test files
        for i in 0..3 {
            let Path = format!("/test_rewind_directory_{i}.txt");

            let test_file = VFS
                .Open(
                    &Path_owned_type::New(Path).unwrap(),
                    File_system::Flags_type::New(
                        Mode_type::WRITE_ONLY,
                        Some(Open_type::CREATE_ONLY),
                        None,
                    ),
                    Task,
                )
                .await
                .unwrap();
            VFS.Close(test_file, Task).await.unwrap();
        }

        // Open directory
        let path = CString::new("/").unwrap();
        let mut directory_id: Xila_unique_file_identifier_type = 0;

        let open_result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening directory should succeed");

        // Read a few entries
        for _ in 0..2 {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
            let mut entry_size: Xila_file_system_size_type = 0;
            let mut entry_inode: Xila_file_system_inode_type = 0;

            let read_result = Context
                .Call_ABI(async || unsafe {
                    Xila_file_system_read_directory(
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
        let rewind_result = Context
            .Call_ABI(|| async { Xila_file_system_rewind_directory(directory_id) })
            .await;
        assert_eq!(rewind_result, 0, "Rewinding directory should succeed");

        // Read first entry again - should be "."
        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
        let mut entry_size: Xila_file_system_size_type = 0;
        let mut entry_inode: Xila_file_system_inode_type = 0;

        let read_result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_read_directory(
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
        let close_result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[Test]
    async fn Test_rewind_directory_invalid_handle() {
        let (_Task, _VFS) = Initialize_test_environment().await;
        let Context = Get_context_instance();

        let invalid_directory_id: Xila_unique_file_identifier_type = 999999;

        let result = Context
            .Call_ABI(|| async { Xila_file_system_rewind_directory(invalid_directory_id) })
            .await;
        assert_ne!(result, 0, "Rewinding invalid directory handle should fail");
    }

    #[Test]
    async fn Test_directory_set_position() {
        let (_Task, VFS) = Initialize_test_environment().await;
        let Task = _Task;
        let Context = Get_context_instance();

        // Create some test files
        for i in 0..5 {
            let Path = format!("/test_file_{i}.txt");

            let test_file = VFS
                .Open(
                    &Path_owned_type::New(Path).unwrap(),
                    File_system::Flags_type::New(
                        Mode_type::WRITE_ONLY,
                        Some(Open_type::CREATE_ONLY),
                        None,
                    ),
                    Task,
                )
                .await
                .unwrap();
            VFS.Close(test_file, Task).await.unwrap();
        }

        // Open directory
        let path = CString::new("/").unwrap();
        let mut directory_id: Xila_unique_file_identifier_type = 0;

        let open_result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening directory should succeed");

        // Read a few entries to advance position
        for _ in 0..3 {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
            let mut entry_size: Xila_file_system_size_type = 0;
            let mut entry_inode: Xila_file_system_inode_type = 0;

            let read_result = Context
                .Call_ABI(async || unsafe {
                    Xila_file_system_read_directory(
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
        let set_position_result = Context
            .Call_ABI(|| async { Xila_file_system_directory_set_position(directory_id, 0) })
            .await;
        assert_eq!(
            set_position_result, 0,
            "Setting directory position should succeed"
        );

        // Read first entry - should be "." again
        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
        let mut entry_size: Xila_file_system_size_type = 0;
        let mut entry_inode: Xila_file_system_inode_type = 0;

        let read_result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_read_directory(
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
        let close_result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[Test]
    async fn Test_directory_set_position_invalid_handle() {
        let (_Task, _VFS) = Initialize_test_environment().await;
        let Context = Get_context_instance();

        let invalid_directory_id: Xila_unique_file_identifier_type = 999999;

        let result = Context
            .Call_ABI(|| async { Xila_file_system_directory_set_position(invalid_directory_id, 0) })
            .await;
        assert_ne!(
            result, 0,
            "Setting position on invalid directory handle should fail"
        );
    }

    #[Test]
    async fn Test_directory_operations_sequence() {
        let (_Task, VFS) = Initialize_test_environment().await;
        let Task = _Task;
        let Context = Get_context_instance();

        // Create test structure
        VFS.Create_directory(&"/test_dir", Task).await.unwrap();
        VFS.Create_directory(&"/test_dir/subdir", Task)
            .await
            .unwrap();

        let test_file = VFS
            .Open(
                &"/test_dir/file.txt",
                File_system::Flags_type::New(
                    Mode_type::WRITE_ONLY,
                    Some(Open_type::CREATE_ONLY),
                    None,
                ),
                Task,
            )
            .await
            .unwrap();
        VFS.Close(test_file, Task).await.unwrap();

        // Test opening the created directory
        let path = CString::new("/test_dir").unwrap();
        let mut directory_id: Xila_unique_file_identifier_type = 0;

        let open_result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(path.as_ptr(), &mut directory_id)
            })
            .await;
        assert_eq!(open_result, 0, "Opening test directory should succeed");

        // Count entries
        let mut entry_count = 0;
        loop {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
            let mut entry_size: Xila_file_system_size_type = 0;
            let mut entry_inode: Xila_file_system_inode_type = 0;

            let read_result = Context
                .Call_ABI(async || unsafe {
                    Xila_file_system_read_directory(
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
        let rewind_result = Context
            .Call_ABI(|| async { Xila_file_system_rewind_directory(directory_id) })
            .await;
        assert_eq!(rewind_result, 0, "Rewinding directory should succeed");

        let mut rewind_count = 0;
        loop {
            let mut entry_name: *const c_char = core::ptr::null();
            let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
            let mut entry_size: Xila_file_system_size_type = 0;
            let mut entry_inode: Xila_file_system_inode_type = 0;

            let read_result = Context
                .Call_ABI(async || unsafe {
                    Xila_file_system_read_directory(
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
        let close_result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(directory_id) })
            .await;
        assert_eq!(close_result, 0, "Closing directory should succeed");
    }

    #[Test]
    async fn Test_directory_operations_error_handling() {
        Initialize_test_environment().await;

        let Context = Get_context_instance();

        // Test null pointer handling
        let mut directory_id: Xila_unique_file_identifier_type = 0;

        // Test with null path
        let result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_open_directory(core::ptr::null(), &mut directory_id)
            })
            .await;
        assert_ne!(result, 0, "Null path should cause error");

        // Test invalid operations on invalid handles
        let invalid_handle = 0usize;

        let close_result = Context
            .Call_ABI(|| async { Xila_file_system_close_directory(invalid_handle) })
            .await;
        assert_ne!(close_result, 0, "Invalid close should fail");

        let rewind_result = Context
            .Call_ABI(|| async { Xila_file_system_rewind_directory(invalid_handle) })
            .await;
        assert_ne!(rewind_result, 0, "Invalid rewind should fail");

        let set_pos_result = Context
            .Call_ABI(|| async { Xila_file_system_directory_set_position(invalid_handle, 0) })
            .await;
        assert_ne!(set_pos_result, 0, "Invalid set position should fail");

        let mut entry_name: *const c_char = core::ptr::null();
        let mut entry_type: Xila_file_type_type = Xila_file_type_type::File;
        let mut entry_size: Xila_file_system_size_type = 0;
        let mut entry_inode: Xila_file_system_inode_type = 0;

        let read_result = Context
            .Call_ABI(async || unsafe {
                Xila_file_system_read_directory(
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
