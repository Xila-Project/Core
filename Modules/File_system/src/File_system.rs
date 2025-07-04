//! Core file system trait and abstractions.
//!
//! This module defines the primary [`File_system_traits`] trait that all file system
//! implementations must implement. It provides a comprehensive POSIX-like interface
//! for file and directory operations with support for multi-user environments,
//! task isolation, and concurrent access.

use alloc::collections::BTreeMap;

use crate::{
    Entry_type, File_identifier_type, Inode_type, Local_file_identifier_type, Metadata_type,
    Mode_type, Statistics_type, Time_type,
};

use super::{Error_type, Flags_type, Path_type, Position_type, Result_type, Size_type};

use Task::Task_identifier_type;
use Users::{Group_identifier_type, User_identifier_type};

/// Convenience macro for creating file system instances.
///
/// This macro wraps file system implementations in a `Box` for dynamic dispatch
/// and easy registration with the virtual file system.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use File_system::*;
///
/// // Create a file system instance (assuming MyFileSystem implements File_system_traits)
/// // let fs = Create_file_system!(MyFileSystem::new());
/// ```
#[macro_export]
macro_rules! Create_file_system {
    ($file_system:expr) => {
        alloc::boxed::Box::new($file_system)
    };
    () => {};
}

/// Core trait for all file system implementations.
///
/// This trait defines the complete interface that file systems must implement to integrate
/// with the Xila operating system. It provides a POSIX-like API with additional features
/// for multi-user environments, task isolation, and modern file system operations.
///
/// ## Design Principles
///
/// ### Task Isolation
/// All file operations are associated with a [`Task_identifier_type`] to ensure proper
/// isolation between processes. File descriptors are local to each task.
///
/// ### User Security
/// Operations include user and group identifiers for permission checking, ensuring
/// secure multi-user operation.
///
/// ### Concurrent Access
/// Implementations must handle concurrent access safely. The trait requires `Send + Sync`
/// and implementations should use appropriate synchronization primitives like `RwLock` or `Mutex`.
///
/// ### Non-blocking Operations
/// All operations should avoid blocking indefinitely. If an operation would block,
/// implementations should return [`Error_type::Ressource_busy`].
///
/// ## File Operations
///
/// The trait supports standard file operations including:
/// - Opening and closing files with various flags
/// - Reading and writing data
/// - Seeking to specific positions
/// - File and directory creation/deletion
/// - Metadata operations
///
/// ## Directory Operations
///
/// Directory operations include:
/// - Creating and removing directories
/// - Opening directories for iteration
/// - Reading directory entries
/// - Position management for directory iteration
///
/// ## Example Implementation Pattern
///
/// ```rust
/// # extern crate alloc;
/// # use File_system::*;
/// # use alloc::collections::BTreeMap;
/// # use Task::Task_identifier_type;
/// # use Users::{User_identifier_type, Group_identifier_type};
/// # use Synchronization::rwlock::RwLock;
/// # use Synchronization::blocking_mutex::raw::CriticalSectionRawMutex;
///
/// struct MyFileSystem {
///     // Use RwLock for thread safety
///     files: RwLock<CriticalSectionRawMutex, BTreeMap<Local_file_identifier_type, u32>>,
///     // ... other fields
/// }
///
/// impl File_system_traits for MyFileSystem {
///     fn Open(&self, task: Task_identifier_type, path: &Path_type, flags: Flags_type,
///             time: Time_type, user: User_identifier_type, group: Group_identifier_type)
///             -> Result_type<Local_file_identifier_type> {
///         todo!()
///     }
///     // ... other methods would be implemented here
/// #    fn Close(&self, _: Local_file_identifier_type) -> Result_type<()> { todo!() }
/// #    fn Close_all(&self, _: Task_identifier_type) -> Result_type<()> { todo!() }
/// #    fn Duplicate(&self, _: Local_file_identifier_type) -> Result_type<Local_file_identifier_type> { todo!() }
/// #    fn Transfert(&self, _: Task_identifier_type, _: Local_file_identifier_type, _: Option<File_identifier_type>) -> Result_type<Local_file_identifier_type> { todo!() }
/// #    fn Remove(&self, _: &Path_type) -> Result_type<()> { todo!() }
/// #    fn Read(&self, _: Local_file_identifier_type, _: &mut [u8], _: Time_type) -> Result_type<Size_type> { todo!() }
/// #    fn Write(&self, _: Local_file_identifier_type, _: &[u8], _: Time_type) -> Result_type<Size_type> { todo!() }
/// #    fn Rename(&self, _: &Path_type, _: &Path_type) -> Result_type<()> { todo!() }
/// #    fn Set_position(&self, _: Local_file_identifier_type, _: &Position_type) -> Result_type<Size_type> { todo!() }
/// #    fn Flush(&self, _: Local_file_identifier_type) -> Result_type<()> { todo!() }
/// #    fn Create_directory(&self, _: &Path_type, _: Time_type, _: User_identifier_type, _: Group_identifier_type) -> Result_type<()> { todo!() }
/// #    fn Open_directory(&self, _: &Path_type, _: Task_identifier_type) -> Result_type<Local_file_identifier_type> { todo!() }
/// #    fn Read_directory(&self, _: Local_file_identifier_type) -> Result_type<Option<Entry_type>> { todo!() }
/// #    fn Set_position_directory(&self, _: Local_file_identifier_type, _: Size_type) -> Result_type<()> { todo!() }
/// #    fn Get_position_directory(&self, _: Local_file_identifier_type) -> Result_type<Size_type> { todo!() }
/// #    fn Rewind_directory(&self, _: Local_file_identifier_type) -> Result_type<()> { todo!() }
/// #    fn Close_directory(&self, _: Local_file_identifier_type) -> Result_type<()> { todo!() }
/// #    fn Get_metadata(&self, _: Local_file_identifier_type) -> Result_type<Metadata_type> { todo!() }
/// #    fn Set_metadata_from_path(&self, _: &Path_type, _: &Metadata_type) -> Result_type<()> { todo!() }
/// #    fn Get_metadata_from_path(&self, _: &Path_type) -> Result_type<Metadata_type> { todo!() }
/// #    fn Get_statistics(&self, _: Local_file_identifier_type) -> Result_type<Statistics_type> { todo!() }
/// #    fn Get_mode(&self, _: Local_file_identifier_type) -> Result_type<Mode_type> { todo!() }
/// }
/// ```
pub trait File_system_traits: Send + Sync {
    // - Status
    // - Manipulation
    // - - Open/close/delete

    /// Open a file for I/O operations.
    ///
    /// Opens a file at the specified path with the given flags and associates it with
    /// the specified task. The operation is subject to permission checking based on
    /// the provided user and group identifiers.
    ///
    /// # Arguments
    ///
    /// * `Task` - Task identifier for file descriptor ownership
    /// * `Path` - Path to the file to open
    /// * `Flags` - Open flags (read, write, create, etc.)
    /// * `Time` - Current time for metadata updates
    /// * `User` - User identifier for permission checking
    /// * `Group` - Group identifier for permission checking
    ///
    /// # Returns
    ///
    /// * `Ok(Local_file_identifier_type)` - File descriptor for the opened file
    /// * `Err(Error_type)` - Error if file cannot be opened
    ///
    /// # Errors
    ///
    /// * [`Error_type::Not_found`] - File doesn't exist and create flag not set
    /// * [`Error_type::Permission_denied`] - Insufficient permissions
    /// * [`Error_type::Already_exists`] - File exists and exclusive create flag set
    /// * [`Error_type::Too_many_open_files`] - File descriptor limit reached
    fn Open(
        &self,
        Task: Task_identifier_type,
        Path: &Path_type,
        Flags: Flags_type,
        Time: Time_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
    ) -> Result_type<Local_file_identifier_type>;

    /// Close a file and release its resources.
    ///
    /// Closes the file associated with the given file identifier and releases any
    /// resources associated with it. After calling this method, the file identifier
    /// becomes invalid for the task.
    ///
    /// # Arguments
    ///
    /// * `File` - File identifier to close
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File successfully closed
    /// * `Err(Error_type)` - Error closing file
    ///
    /// # Errors
    ///
    /// * [`Error_type::Invalid_identifier`] - File identifier is invalid
    /// * [`Error_type::Input_output`] - I/O error during close operation
    fn Close(&self, File: Local_file_identifier_type) -> Result_type<()>;

    /// Close all files opened by a specific task.
    ///
    /// This is typically called during task cleanup to ensure all file descriptors
    /// are properly released when a task terminates.
    ///
    /// # Arguments
    ///
    /// * `Task` - Task identifier whose files should be closed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All files successfully closed
    /// * `Err(Error_type)` - Error during cleanup
    fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()>;

    /// Create a duplicate file identifier for the same file.
    ///
    /// Creates a new file identifier that refers to the same open file. This is
    /// similar to the `dup()` system call in Unix systems. Both identifiers
    /// can be used independently and must be closed separately.
    ///
    /// # Arguments
    ///
    /// * `File` - File identifier to duplicate
    ///
    /// # Returns
    ///
    /// * `Ok(Local_file_identifier_type)` - New file identifier for the same file
    /// * `Err(Error_type)` - Error creating duplicate
    ///
    /// # Errors
    ///
    /// * [`Error_type::Invalid_identifier`] - Original file identifier is invalid
    /// * [`Error_type::Too_many_open_files`] - File descriptor limit reached
    fn Duplicate(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type>;

    /// Transfer a file identifier from one task to another.
    ///
    /// Moves ownership of a file identifier from the current task to another task.
    /// This is useful for inter-process communication and file descriptor passing.
    ///
    /// # Arguments
    ///
    /// * `New_task` - Task to transfer the file to
    /// * `File` - File identifier to transfer
    /// * `New_file` - Optional specific identifier to use in the new task
    ///
    /// # Returns
    ///
    /// * `Ok(Local_file_identifier_type)` - File identifier in the new task
    /// * `Err(Error_type)` - Error during transfer
    ///
    /// # Errors
    ///
    /// * [`Error_type::Invalid_identifier`] - File identifier is invalid
    /// * [`Error_type::Failed_to_get_task_informations`] - Target task is invalid
    fn Transfert(
        &self,
        New_task: Task_identifier_type,
        File: Local_file_identifier_type,
        New_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type>;

    /// Remove a file or directory from the file system.
    ///
    /// Permanently deletes the specified file or directory. For directories,
    /// they must be empty before they can be removed.
    ///
    /// # Arguments
    ///
    /// * `Path` - Path to the file or directory to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File or directory successfully removed
    /// * `Err(Error_type)` - Error during removal
    ///
    /// # Errors
    ///
    /// * [`Error_type::Not_found`] - File or directory doesn't exist
    /// * [`Error_type::Permission_denied`] - Insufficient permissions
    /// * [`Error_type::Directory_not_empty`] - Directory contains files
    /// * [`Error_type::Ressource_busy`] - File is currently in use
    fn Remove(&self, Path: &Path_type) -> Result_type<()>;
    // - - File operations

    /// Read data from an open file.
    ///
    /// Reads data from the file at its current position into the provided buffer.
    /// The file position is advanced by the number of bytes read.
    ///
    /// # Arguments
    ///
    /// * `File` - File identifier to read from
    /// * `Buffer` - Buffer to read data into
    /// * `Time_type` - Current time for access time updates
    ///
    /// # Returns
    ///
    /// * `Ok(Size_type)` - Number of bytes actually read
    /// * `Err(Error_type)` - Error during read operation
    ///
    /// # Errors
    ///
    /// * [`Error_type::Invalid_identifier`] - File identifier is invalid
    /// * [`Error_type::Permission_denied`] - File not opened for reading
    /// * [`Error_type::Input_output`] - I/O error during read
    fn Read(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut [u8],
        Time_type: Time_type,
    ) -> Result_type<Size_type>;

    /// Write data to an open file.
    ///
    /// Writes data from the buffer to the file at its current position.
    /// The file position is advanced by the number of bytes written.
    ///
    /// # Arguments
    ///
    /// * `File` - File identifier to write to
    /// * `Buffer` - Buffer containing data to write
    /// * `Time_type` - Current time for modification time updates
    ///
    /// # Returns
    ///
    /// * `Ok(Size_type)` - Number of bytes actually written
    /// * `Err(Error_type)` - Error during write operation
    ///
    /// # Errors
    ///
    /// * [`Error_type::Invalid_identifier`] - File identifier is invalid
    /// * [`Error_type::Permission_denied`] - File not opened for writing
    /// * [`Error_type::No_space_left`] - Insufficient storage space
    /// * [`Error_type::Input_output`] - I/O error during write
    fn Write(
        &self,
        File: Local_file_identifier_type,
        Buffer: &[u8],
        Time_type: Time_type,
    ) -> Result_type<Size_type>;

    /// Rename or move a file or directory.
    ///
    /// Changes the name or location of a file or directory. This can be used
    /// for both renaming within the same directory and moving between directories.
    ///
    /// # Arguments
    ///
    /// * `Source` - Current path of the file or directory
    /// * `Destination` - New path for the file or directory
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File or directory successfully renamed/moved
    /// * `Err(Error_type)` - Error during rename operation
    ///
    /// # Errors
    ///
    /// * [`Error_type::Not_found`] - Source file doesn't exist
    /// * [`Error_type::Already_exists`] - Destination already exists
    /// * [`Error_type::Permission_denied`] - Insufficient permissions
    fn Rename(&self, Source: &Path_type, Destination: &Path_type) -> Result_type<()>;

    /// Set the position of the file.
    ///
    /// # Errors
    /// - If the file is not opened (invalid file identifier).
    fn Set_position(
        &self,
        File: Local_file_identifier_type,
        Position: &Position_type,
    ) -> Result_type<Size_type>;

    fn Flush(&self, File: Local_file_identifier_type) -> Result_type<()>;

    // - Directory

    fn Create_directory(
        &self,
        Path: &Path_type,
        Time: Time_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
    ) -> Result_type<()>;

    fn Open_directory(
        &self,
        Path: &Path_type,
        Task: Task_identifier_type,
    ) -> Result_type<Local_file_identifier_type>;

    fn Read_directory(&self, File: Local_file_identifier_type) -> Result_type<Option<Entry_type>>;

    fn Set_position_directory(
        &self,
        File: Local_file_identifier_type,
        Position: Size_type,
    ) -> Result_type<()>;

    fn Get_position_directory(&self, File: Local_file_identifier_type) -> Result_type<Size_type>;

    fn Rewind_directory(&self, File: Local_file_identifier_type) -> Result_type<()>;

    fn Close_directory(&self, File: Local_file_identifier_type) -> Result_type<()>;

    // - Metadata

    fn Get_metadata(&self, File: Local_file_identifier_type) -> Result_type<Metadata_type>;

    fn Set_metadata_from_path(&self, Path: &Path_type, Metadata: &Metadata_type)
        -> Result_type<()>;

    fn Get_metadata_from_path(&self, Path: &Path_type) -> Result_type<Metadata_type>;

    fn Get_statistics(&self, File: Local_file_identifier_type) -> Result_type<Statistics_type>;

    fn Get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type>;
}

pub fn Get_new_file_identifier<T>(
    Task: Task_identifier_type,
    Start: Option<File_identifier_type>,
    End: Option<File_identifier_type>,
    Map: &BTreeMap<Local_file_identifier_type, T>,
) -> Result_type<Local_file_identifier_type> {
    let Start = Start.unwrap_or(File_identifier_type::Minimum);
    let mut Start = Local_file_identifier_type::New(Task, Start);

    let End = End.unwrap_or(File_identifier_type::Maximum);
    let End = Local_file_identifier_type::New(Task, End);

    while Start < End {
        if !Map.contains_key(&Start) {
            return Ok(Start);
        }

        Start += 1;
    }

    Err(Error_type::Too_many_open_files)
}

pub fn Get_new_inode<T>(Map: &BTreeMap<Inode_type, T>) -> Result_type<Inode_type> {
    let mut Inode = Inode_type::from(0);

    while Map.contains_key(&Inode) {
        Inode += 1;
    }

    Ok(Inode)
}

pub mod Tests {

    use crate::{Open_type, Path_owned_type, Time_type, Type_type};

    use alloc::{borrow::ToOwned, format};

    use super::*;

    pub fn Get_test_path() -> Path_owned_type {
        Path_type::Root.to_owned()
    }

    pub async fn Test_open_close_delete(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Path = Get_test_path().Append("Test_open_close_delete").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system
            .Open(
                Task,
                &Path,
                Flags,
                Time_type::New(123),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Remove(&Path).unwrap();
    }

    pub async fn Test_read_write(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Path = Get_test_path().Append("Test_read_write").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system
            .Open(
                Task,
                &Path,
                Flags,
                Time_type::New(123),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system
            .Write(File, &Buffer, Time_type::New(123))
            .unwrap();

        assert_eq!(Size, Size_type::from(Buffer.len()));
        File_system
            .Set_position(File, &Position_type::Start(0))
            .unwrap();

        // - Read
        let mut Buffer_read = [0; 3];
        let Size = File_system
            .Read(File, &mut Buffer_read, Time_type::New(123))
            .unwrap();
        assert_eq!(Buffer, Buffer_read);
        assert_eq!(Size, Size_type::from(Buffer.len()));

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Remove(&Path).unwrap();
    }

    pub async fn Test_move(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Path = Get_test_path().Append("Test_move").unwrap();
        let Path_destination = Get_test_path().Append("Test_move_destination").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system
            .Open(
                Task,
                &Path,
                Flags,
                Time_type::New(123),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system
            .Write(File, &Buffer, Time_type::New(123))
            .unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));

        File_system.Close(File).unwrap();

        // - Move
        File_system.Rename(&Path, &Path_destination).unwrap();

        // - Open
        let File = File_system
            .Open(
                Task,
                &Path_destination,
                Mode_type::Read_only.into(),
                Time_type::New(123),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        // - Read
        let mut Buffer_read = [0; 3];
        let Size = File_system
            .Read(File, &mut Buffer_read, Time_type::New(123))
            .unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));
        assert_eq!(Buffer, Buffer_read);

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Remove(&Path_destination).unwrap();
    }

    pub async fn Test_set_position(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Path = Get_test_path().Append("Test_set_position").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system
            .Open(
                Task,
                &Path,
                Flags,
                Time_type::New(123),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system
            .Write(File, &Buffer, Time_type::New(123))
            .unwrap();
        assert_eq!(Buffer.len(), Size.into());

        // - Set position
        let Position = Position_type::Start(0);
        let Size = File_system.Set_position(File, &Position).unwrap();
        assert_eq!(
            Size,
            File_system
                .Set_position(File, &Position_type::Current(0))
                .unwrap()
        );

        // - Read
        let mut Buffer_read = [0; 3];
        let Size = File_system
            .Read(File, &mut Buffer_read, Time_type::New(123))
            .unwrap();
        assert_eq!(Buffer, Buffer_read);
        assert_eq!(Buffer.len(), Size.into());

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Remove(&Path).unwrap();
    }

    pub async fn Test_flush(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Path = Get_test_path().Append("Test_flush").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        let File = File_system
            .Open(
                Task,
                &Path,
                Flags,
                Time_type::New(123),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system
            .Write(File, &Buffer, Time_type::New(123))
            .unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));

        File_system.Flush(File).unwrap();

        File_system.Close(File).unwrap();

        File_system.Remove(&Path).unwrap();
    }

    pub async fn Test_set_get_metadata(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Path = Get_test_path().Append("Test_set_owner").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        let File = File_system
            .Open(
                Task,
                &Path,
                Flags,
                Time_type::New(123),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        let Time = Time_type::New(123);

        let Metadata = Metadata_type::Get_default(
            Type_type::File,
            Time,
            User_identifier_type::Root,
            Group_identifier_type::Root,
        )
        .unwrap();

        File_system
            .Set_metadata_from_path(&Path, &Metadata)
            .unwrap();

        let Metadata_read = File_system.Get_metadata_from_path(&Path).unwrap();

        assert_eq!(Metadata, Metadata_read);

        File_system.Close(File).unwrap();

        File_system.Remove(&Path).unwrap();
    }

    pub async fn Test_read_directory(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        // Create multiple files
        for i in 0..10 {
            let Flags = Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None);
            let File = File_system
                .Open(
                    Task,
                    Path_type::From_str(&format!("/Test{i}")),
                    Flags,
                    Time_type::New(123),
                    User_identifier_type::Root,
                    Group_identifier_type::Root,
                )
                .unwrap();
            File_system.Close(File).unwrap();
        }

        let Path = Path_type::From_str("/");
        let Directory = File_system.Open_directory(Path, Task).unwrap();

        let Current_directory = File_system.Read_directory(Directory).unwrap().unwrap();
        assert_eq!(*Current_directory.Get_name(), ".");
        assert_eq!(Current_directory.Get_type(), Type_type::Directory);

        let Parent_directory = File_system.Read_directory(Directory).unwrap().unwrap();
        assert_eq!(*Parent_directory.Get_name(), "..");
        assert_eq!(Parent_directory.Get_type(), Type_type::Directory);

        for i in 0..10 {
            let Entry = File_system.Read_directory(Directory).unwrap().unwrap();

            assert_eq!(*Entry.Get_name(), format!("Test{i}"));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }

        File_system.Close_directory(Directory).unwrap();
    }

    pub async fn Test_set_position_directory(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        // Create multiple files
        for i in 0..10 {
            let Flags = Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None);
            let File = File_system
                .Open(
                    Task,
                    Path_type::From_str(&format!("/Test{i}")),
                    Flags,
                    Time_type::New(123),
                    User_identifier_type::Root,
                    Group_identifier_type::Root,
                )
                .unwrap();
            File_system.Close(File).unwrap();
        }

        let Directory = File_system.Open_directory(Path_type::Root, Task).unwrap();

        let Current_directory = File_system.Read_directory(Directory).unwrap().unwrap();
        assert_eq!(*Current_directory.Get_name(), ".");
        assert_eq!(Current_directory.Get_type(), Type_type::Directory);

        let Parent_directory = File_system.Read_directory(Directory).unwrap().unwrap();
        assert_eq!(*Parent_directory.Get_name(), "..");
        assert_eq!(Parent_directory.Get_type(), Type_type::Directory);

        let Position = File_system.Get_position_directory(Directory).unwrap();

        for i in 0..10 {
            let Entry = File_system.Read_directory(Directory).unwrap().unwrap();

            assert_eq!(*Entry.Get_name(), format!("Test{i}"));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }

        File_system
            .Set_position_directory(Directory, Position)
            .unwrap();

        for i in 0..10 {
            let Entry = File_system.Read_directory(Directory).unwrap().unwrap();

            assert_eq!(*Entry.Get_name(), format!("Test{i}"));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }
    }

    pub async fn Test_rewind_directory(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        // Create multiple files
        for i in 0..10 {
            let Flags = Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None);
            let File = File_system
                .Open(
                    Task,
                    Path_type::From_str(&format!("/Test{i}")),
                    Flags,
                    Time_type::New(123),
                    User_identifier_type::Root,
                    Group_identifier_type::Root,
                )
                .unwrap();
            File_system.Close(File).unwrap();
        }

        let Directory = File_system.Open_directory(Path_type::Root, Task).unwrap();

        let Current_directory = File_system.Read_directory(Directory).unwrap().unwrap();
        assert_eq!(*Current_directory.Get_name(), ".");
        assert_eq!(Current_directory.Get_type(), Type_type::Directory);

        let Parent_directory = File_system.Read_directory(Directory).unwrap().unwrap();
        assert_eq!(*Parent_directory.Get_name(), "..");
        assert_eq!(Parent_directory.Get_type(), Type_type::Directory);

        for i in 0..10 {
            let Entry = File_system.Read_directory(Directory).unwrap().unwrap();

            assert_eq!(*Entry.Get_name(), format!("Test{i}"));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }

        File_system.Rewind_directory(Directory).unwrap();

        let Current_directory = File_system.Read_directory(Directory).unwrap().unwrap();
        assert_eq!(*Current_directory.Get_name(), ".");
        assert_eq!(Current_directory.Get_type(), Type_type::Directory);

        let Parent_directory = File_system.Read_directory(Directory).unwrap().unwrap();
        assert_eq!(*Parent_directory.Get_name(), "..");
        assert_eq!(Parent_directory.Get_type(), Type_type::Directory);

        for i in 0..10 {
            let Entry = File_system.Read_directory(Directory).unwrap().unwrap();

            assert_eq!(*Entry.Get_name(), format!("Test{i}"));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }

        File_system.Close_directory(Directory).unwrap();
    }

    pub async fn Test_create_remove_directory(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().await;

        let Path = Get_test_path().Append("Test_create_directory").unwrap();

        File_system
            .Create_directory(
                &Path,
                Time_type::New(123),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        {
            let Root_directory = File_system.Open_directory(Path_type::Root, Task).unwrap();

            let Current_directory = File_system.Read_directory(Root_directory).unwrap().unwrap();
            assert_eq!(*Current_directory.Get_name(), ".");
            assert_eq!(Current_directory.Get_type(), Type_type::Directory);

            let Parent_directory = File_system.Read_directory(Root_directory).unwrap().unwrap();
            assert_eq!(*Parent_directory.Get_name(), "..");
            assert_eq!(Parent_directory.Get_type(), Type_type::Directory);

            let Directory = File_system.Read_directory(Root_directory).unwrap().unwrap();
            assert_eq!(*Directory.Get_name(), "Test_create_directory");
            assert_eq!(Directory.Get_type(), Type_type::Directory);

            File_system.Close_directory(Root_directory).unwrap();
        }

        {
            let Directory = File_system.Open_directory(&Path, Task).unwrap();

            let Current_directory = File_system.Read_directory(Directory).unwrap().unwrap();

            assert_eq!(*Current_directory.Get_name(), ".");
            assert_eq!(Current_directory.Get_type(), Type_type::Directory);

            let Parent_directory = File_system.Read_directory(Directory).unwrap().unwrap();
            assert_eq!(*Parent_directory.Get_name(), "..");
            assert_eq!(Parent_directory.Get_type(), Type_type::Directory);

            File_system.Close_directory(Directory).unwrap();
        }
        File_system.Remove(&Path).unwrap();
    }
}
