//! Core file system trait and abstractions.
//!
//! This module defines the primary [`FileSystemTraits`] trait that all file system
//! implementations must implement. It provides a comprehensive POSIX-like interface
//! for file and directory operations with support for multi-user environments,
//! task isolation, and concurrent access.

use alloc::collections::BTreeMap;

use crate::{
    Entry, FileIdentifier, Inode, LocalFileIdentifier, Metadata, Mode, Statistics_type, Time,
};

use super::{Error, Flags, Path, Position, Result, Size};

use task::TaskIdentifier;
use users::{GroupIdentifier, UserIdentifier};

/// Convenience macro for creating file system instances.
///
/// This macro wraps file system implementations in a `Box` for dynamic dispatch
/// and easy registration with the virtual file system.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use file_system::*;
///
/// // Create a file system instance (assuming MyFileSystem implements FileSystemTraits)
/// // let fs = Create_file_system!(MyFileSystem::new());
/// ```
#[macro_export]
macro_rules! create_file_system {
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
/// All file operations are associated with a [`TaskIdentifier`] to ensure proper
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
/// implementations should return [`Error::RessourceBusy`].
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
/// # use file_system::*;
/// # use alloc::collections::BTreeMap;
/// # use task::TaskIdentifier;
/// # use users::{UserIdentifier, GroupIdentifier};
/// # use synchronization::rwlock::RwLock;
/// # use synchronization::blocking_mutex::raw::CriticalSectionRawMutex;
///
/// struct MyFileSystem {
///     // Use RwLock for thread safety
///     files: RwLock<CriticalSectionRawMutex, BTreeMap<LocalFileIdentifier, u32>>,
///     // ... other fields
/// }
///
/// impl FileSystemTraits for MyFileSystem {
///     fn open(&self, task: TaskIdentifier, path: &Path, flags: Flags,
///             time: Time, user: UserIdentifier, group: GroupIdentifier)
///             -> Result<LocalFileIdentifier> {
///         todo!()
///     }
///     // ... other methods would be implemented here
/// #    fn close(&self, _: LocalFileIdentifier) -> Result<()> { todo!() }
/// #    fn close_all(&self, _: TaskIdentifier) -> Result<()> { todo!() }
/// #    fn duplicate(&self, _: LocalFileIdentifier) -> Result<LocalFileIdentifier> { todo!() }
/// #    fn transfert(&self, _: TaskIdentifier, _: LocalFileIdentifier, _: Option<FileIdentifier>) -> Result<LocalFileIdentifier> { todo!() }
/// #    fn remove(&self, _: &Path) -> Result<()> { todo!() }
/// #    fn read(&self, _: LocalFileIdentifier, _: &mut [u8], _: Time) -> Result<Size> { todo!() }
/// #    fn write(&self, _: LocalFileIdentifier, _: &[u8], _: Time) -> Result<Size> { todo!() }
/// #    fn rename(&self, _: &Path, _: &Path) -> Result<()> { todo!() }
/// #    fn set_position(&self, _: LocalFileIdentifier, _: &Position) -> Result<Size> { todo!() }
/// #    fn flush(&self, _: LocalFileIdentifier) -> Result<()> { todo!() }
/// #    fn create_directory(&self, _: &Path, _: Time, _: UserIdentifier, _: GroupIdentifier) -> Result<()> { todo!() }
/// #    fn open_directory(&self, _: &Path, _: TaskIdentifier) -> Result<LocalFileIdentifier> { todo!() }
/// #    fn read_directory(&self, _: LocalFileIdentifier) -> Result<Option<Entry>> { todo!() }
/// #    fn set_position_directory(&self, _: LocalFileIdentifier, _: Size) -> Result<()> { todo!() }
/// #    fn get_position_directory(&self, _: LocalFileIdentifier) -> Result<Size> { todo!() }
/// #    fn rewind_directory(&self, _: LocalFileIdentifier) -> Result<()> { todo!() }
/// #    fn close_directory(&self, _: LocalFileIdentifier) -> Result<()> { todo!() }
/// #    fn get_metadata(&self, _: LocalFileIdentifier) -> Result<Metadata> { todo!() }
/// #    fn set_metadata_from_path(&self, _: &Path, _: &Metadata) -> Result<()> { todo!() }
/// #    fn get_metadata_from_path(&self, _: &Path) -> Result<Metadata> { todo!() }
/// #    fn get_statistics(&self, _: LocalFileIdentifier) -> Result<Statistics_type> { todo!() }
/// #    fn get_mode(&self, _: LocalFileIdentifier) -> Result<Mode> { todo!() }
/// }
/// ```
pub trait FileSystemTraits: Send + Sync {
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
    /// * `Ok(LocalFileIdentifier)` - File descriptor for the opened file
    /// * `Err(Error)` - Error if file cannot be opened
    ///
    /// # Errors
    ///
    /// * [`Error::NotFound`] - File doesn't exist and create flag not set
    /// * [`Error::PermissionDenied`] - Insufficient permissions
    /// * [`Error::AlreadyExists`] - File exists and exclusive create flag set
    /// * [`Error::TooManyOpenFiles`] - File descriptor limit reached
    fn open(
        &self,
        task: TaskIdentifier,
        path: &Path,
        flags: Flags,
        time: Time,
        user: UserIdentifier,
        group: GroupIdentifier,
    ) -> Result<LocalFileIdentifier>;

    /// Close a file and release its resources.
    ///
    /// Closes the file associated with the given file identifier and releases any
    /// resources associated with it. After calling this method, the file identifier
    /// becomes invalid for the task.
    ///
    /// # Arguments
    ///
    /// * `file` - File identifier to close
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File successfully closed
    /// * `Err(Error)` - Error closing file
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidIdentifier`] - File identifier is invalid
    /// * [`Error::InputOutput`] - I/O error during close operation
    fn close(&self, file: LocalFileIdentifier) -> Result<()>;

    /// Close all files opened by a specific task.
    ///
    /// This is typically called during task cleanup to ensure all file descriptors
    /// are properly released when a task terminates.
    ///
    /// # Arguments
    ///
    /// * `task` - Task identifier whose files should be closed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All files successfully closed
    /// * `Err(Error)` - Error during cleanup
    fn close_all(&self, task: TaskIdentifier) -> Result<()>;

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
    /// * `Ok(LocalFileIdentifier)` - New file identifier for the same file
    /// * `Err(Error)` - Error creating duplicate
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidIdentifier`] - Original file identifier is invalid
    /// * [`Error::TooManyOpenFiles`] - File descriptor limit reached
    fn duplicate(&self, file: LocalFileIdentifier) -> Result<LocalFileIdentifier>;

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
    /// * `Ok(LocalFileIdentifier)` - File identifier in the new task
    /// * `Err(Error)` - Error during transfer
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidIdentifier`] - File identifier is invalid
    /// * [`Error::FailedToGetTaskInformations`] - Target task is invalid
    fn transfer(
        &self,
        new_task: TaskIdentifier,
        file: LocalFileIdentifier,
        new_file: Option<FileIdentifier>,
    ) -> Result<LocalFileIdentifier>;

    /// Remove a file or directory from the file system.
    ///
    /// Permanently deletes the specified file or directory. For directories,
    /// they must be empty before they can be removed.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file or directory to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File or directory successfully removed
    /// * `Err(Error)` - Error during removal
    ///
    /// # Errors
    ///
    /// * [`Error::NotFound`] - File or directory doesn't exist
    /// * [`Error::PermissionDenied`] - Insufficient permissions
    /// * [`Error::DirectoryNotEmpty`] - Directory contains files
    /// * [`Error::RessourceBusy`] - File is currently in use
    fn remove(&self, path: &Path) -> Result<()>;
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
    /// * `Time` - Current time for access time updates
    ///
    /// # Returns
    ///
    /// * `Ok(Size)` - Number of bytes actually read
    /// * `Err(Error)` - Error during read operation
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidIdentifier`] - File identifier is invalid
    /// * [`Error::PermissionDenied`] - File not opened for reading
    /// * [`Error::InputOutput`] - I/O error during read
    fn read(&self, file: LocalFileIdentifier, buffer: &mut [u8], time_type: Time) -> Result<Size>;

    /// Write data to an open file.
    ///
    /// Writes data from the buffer to the file at its current position.
    /// The file position is advanced by the number of bytes written.
    ///
    /// # Arguments
    ///
    /// * `File` - File identifier to write to
    /// * `Buffer` - Buffer containing data to write
    /// * `Time` - Current time for modification time updates
    ///
    /// # Returns
    ///
    /// * `Ok(Size)` - Number of bytes actually written
    /// * `Err(Error)` - Error during write operation
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidIdentifier`] - File identifier is invalid
    /// * [`Error::PermissionDenied`] - File not opened for writing
    /// * [`Error::NoSpaceLeft`] - Insufficient storage space
    /// * [`Error::InputOutput`] - I/O error during write
    fn write(&self, file: LocalFileIdentifier, buffer: &[u8], time_type: Time) -> Result<Size>;

    /// Rename or move a file or directory.
    ///
    /// Changes the name or location of a file or directory. This can be used
    /// for both renaming within the same directory and moving between directories.
    ///
    /// # Arguments
    ///
    /// * `source` - Current path of the file or directory
    /// * `destination` - New path for the file or directory
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File or directory successfully renamed/moved
    /// * `Err(Error)` - Error during rename operation
    ///
    /// # Errors
    ///
    /// * [`Error::NotFound`] - Source file doesn't exist
    /// * [`Error::AlreadyExists`] - Destination already exists
    /// * [`Error::PermissionDenied`] - Insufficient permissions
    fn rename(&self, source: &Path, destination: &Path) -> Result<()>;

    /// Set the position of the file.
    ///
    /// # Errors
    /// - If the file is not opened (invalid file identifier).
    fn set_position(&self, file: LocalFileIdentifier, position: &Position) -> Result<Size>;

    fn flush(&self, file: LocalFileIdentifier) -> Result<()>;

    // - Directory

    fn create_directory(
        &self,
        path: &Path,
        time: Time,
        user: UserIdentifier,
        group: GroupIdentifier,
    ) -> Result<()>;

    fn open_directory(&self, path: &Path, task: TaskIdentifier) -> Result<LocalFileIdentifier>;

    fn read_directory(&self, file: LocalFileIdentifier) -> Result<Option<Entry>>;

    fn set_position_directory(&self, file: LocalFileIdentifier, position: Size) -> Result<()>;

    fn get_position_directory(&self, file: LocalFileIdentifier) -> Result<Size>;

    fn rewind_directory(&self, file: LocalFileIdentifier) -> Result<()>;

    fn close_directory(&self, file: LocalFileIdentifier) -> Result<()>;

    // - Metadata

    fn get_metadata(&self, file: LocalFileIdentifier) -> Result<Metadata>;

    fn set_metadata_from_path(&self, path: &Path, metadata: &Metadata) -> Result<()>;

    fn get_metadata_from_path(&self, path: &Path) -> Result<Metadata>;

    fn get_statistics(&self, file: LocalFileIdentifier) -> Result<Statistics_type>;

    fn get_mode(&self, file: LocalFileIdentifier) -> Result<Mode>;
}

pub fn get_new_file_identifier<T>(
    task: TaskIdentifier,
    start: Option<FileIdentifier>,
    end: Option<FileIdentifier>,
    map: &BTreeMap<LocalFileIdentifier, T>,
) -> Result<LocalFileIdentifier> {
    let start = start.unwrap_or(FileIdentifier::MINIMUM);
    let mut start = LocalFileIdentifier::new(task, start);

    let end_value = end.unwrap_or(FileIdentifier::MAXIMUM);
    let end = LocalFileIdentifier::new(task, end_value);

    while start < end {
        if !map.contains_key(&start) {
            return Ok(start);
        }

        start += 1;
    }

    Err(Error::TooManyOpenFiles)
}

pub fn get_new_inode<T>(map: &BTreeMap<Inode, T>) -> Result<Inode> {
    let mut inode = Inode::from(0);

    while map.contains_key(&inode) {
        inode += 1;
    }

    Ok(inode)
}

pub mod tests {

    use crate::{Kind, Open, PathOwned, Time};

    use alloc::{borrow::ToOwned, format};

    use super::*;

    pub fn get_test_path() -> PathOwned {
        Path::ROOT.to_owned()
    }

    pub async fn test_open_close_delete(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        let path = get_test_path().append("Test_open_close_delete").unwrap();

        let flags = Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None);

        // - Open
        let file = file_system
            .open(
                task,
                &path,
                flags,
                Time::new(123),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        // - Close
        file_system.close(file).unwrap();

        // - Delete
        file_system.remove(&path).unwrap();
    }

    pub async fn test_read_write(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        let path = get_test_path().append("Test_read_write").unwrap();

        let flags = Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None);

        // - Open
        let file = file_system
            .open(
                task,
                &path,
                flags,
                Time::new(123),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        // - Write
        let buffer = [0x01, 0x02, 0x03];
        let size = file_system.write(file, &buffer, Time::new(123)).unwrap();

        assert_eq!(size, Size::from(buffer.len()));
        file_system.set_position(file, &Position::Start(0)).unwrap();

        // - Read
        let mut buffer_read = [0; 3];
        let size = file_system
            .read(file, &mut buffer_read, Time::new(123))
            .unwrap();
        assert_eq!(buffer, buffer_read);
        assert_eq!(size, Size::from(buffer.len()));

        // - Close
        file_system.close(file).unwrap();

        // - Delete
        file_system.remove(&path).unwrap();
    }

    pub async fn test_move(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        let path = get_test_path().append("Test_move").unwrap();
        let path_destination = get_test_path().append("Test_move_destination").unwrap();

        let flags = Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None);

        // - Open
        let file = file_system
            .open(
                task,
                &path,
                flags,
                Time::new(123),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        // - Write
        let buffer = [0x01, 0x02, 0x03];
        let size = file_system.write(file, &buffer, Time::new(123)).unwrap();
        assert_eq!(size, Size::from(buffer.len()));

        file_system.close(file).unwrap();

        // - Move
        file_system.rename(&path, &path_destination).unwrap();

        // - Open
        let file = file_system
            .open(
                task,
                &path_destination,
                Mode::READ_ONLY.into(),
                Time::new(123),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        // - Read
        let mut buffer_read = [0; 3];
        let size = file_system
            .read(file, &mut buffer_read, Time::new(123))
            .unwrap();
        assert_eq!(size, Size::from(buffer.len()));
        assert_eq!(buffer, buffer_read);

        // - Close
        file_system.close(file).unwrap();

        // - Delete
        file_system.remove(&path_destination).unwrap();
    }

    pub async fn test_set_position(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        let path = get_test_path().append("Test_set_position").unwrap();

        let flags = Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None);

        // - Open
        let file = file_system
            .open(
                task,
                &path,
                flags,
                Time::new(123),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        // - Write
        let buffer = [0x01, 0x02, 0x03];
        let size = file_system.write(file, &buffer, Time::new(123)).unwrap();
        assert_eq!(buffer.len(), size.into());

        // - Set position
        let position = Position::Start(0);
        let size = file_system.set_position(file, &position).unwrap();
        assert_eq!(
            size,
            file_system
                .set_position(file, &Position::Current(0))
                .unwrap()
        );

        // - Read
        let mut buffer_read = [0; 3];
        let size = file_system
            .read(file, &mut buffer_read, Time::new(123))
            .unwrap();
        assert_eq!(buffer, buffer_read);
        assert_eq!(buffer.len(), size.into());

        // - Close
        file_system.close(file).unwrap();

        // - Delete
        file_system.remove(&path).unwrap();
    }

    pub async fn test_flush(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        let path = get_test_path().append("Test_flush").unwrap();

        let flags = Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None);

        let file = file_system
            .open(
                task,
                &path,
                flags,
                Time::new(123),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        let buffer = [0x01, 0x02, 0x03];
        let size = file_system.write(file, &buffer, Time::new(123)).unwrap();
        assert_eq!(size, Size::from(buffer.len()));

        file_system.flush(file).unwrap();

        file_system.close(file).unwrap();

        file_system.remove(&path).unwrap();
    }

    pub async fn test_set_get_metadata(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        let path = get_test_path().append("Test_set_owner").unwrap();

        let flags = Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None);

        let file = file_system
            .open(
                task,
                &path,
                flags,
                Time::new(123),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        let time = Time::new(123);

        let metadata = Metadata::get_default(
            Kind::File,
            time,
            UserIdentifier::ROOT,
            GroupIdentifier::ROOT,
        )
        .unwrap();

        file_system
            .set_metadata_from_path(&path, &metadata)
            .unwrap();

        let metadata_read = file_system.get_metadata_from_path(&path).unwrap();

        assert_eq!(metadata, metadata_read);

        file_system.close(file).unwrap();

        file_system.remove(&path).unwrap();
    }

    pub async fn test_read_directory(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        // Create multiple files
        for i in 0..10 {
            let flags = Flags::new(Mode::WRITE_ONLY, Some(Open::CREATE_ONLY), None);
            let file = file_system
                .open(
                    task,
                    Path::from_str(&format!("/Test{i}")),
                    flags,
                    Time::new(123),
                    UserIdentifier::ROOT,
                    GroupIdentifier::ROOT,
                )
                .unwrap();
            file_system.close(file).unwrap();
        }

        let path = Path::from_str("/");
        let directory = file_system.open_directory(path, task).unwrap();

        let current_directory = file_system.read_directory(directory).unwrap().unwrap();
        assert_eq!(*current_directory.get_name(), ".");
        assert_eq!(current_directory.get_type(), Kind::Directory);

        let parent_directory = file_system.read_directory(directory).unwrap().unwrap();
        assert_eq!(*parent_directory.get_name(), "..");
        assert_eq!(parent_directory.get_type(), Kind::Directory);

        for i in 0..10 {
            let entry = file_system.read_directory(directory).unwrap().unwrap();

            assert_eq!(*entry.get_name(), format!("Test{i}"));
            assert_eq!(entry.get_type(), Kind::File);
        }

        file_system.close_directory(directory).unwrap();
    }

    pub async fn test_set_position_directory(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        // Create multiple files
        for i in 0..10 {
            let flags = Flags::new(Mode::WRITE_ONLY, Some(Open::CREATE_ONLY), None);
            let file = file_system
                .open(
                    task,
                    Path::from_str(&format!("/Test{i}")),
                    flags,
                    Time::new(123),
                    UserIdentifier::ROOT,
                    GroupIdentifier::ROOT,
                )
                .unwrap();
            file_system.close(file).unwrap();
        }

        let directory = file_system.open_directory(Path::ROOT, task).unwrap();

        let current_directory = file_system.read_directory(directory).unwrap().unwrap();
        assert_eq!(*current_directory.get_name(), ".");
        assert_eq!(current_directory.get_type(), Kind::Directory);

        let parent_directory = file_system.read_directory(directory).unwrap().unwrap();
        assert_eq!(*parent_directory.get_name(), "..");
        assert_eq!(parent_directory.get_type(), Kind::Directory);

        let position = file_system.get_position_directory(directory).unwrap();

        for i in 0..10 {
            let entry = file_system.read_directory(directory).unwrap().unwrap();

            assert_eq!(*entry.get_name(), format!("Test{i}"));
            assert_eq!(entry.get_type(), Kind::File);
        }

        file_system
            .set_position_directory(directory, position)
            .unwrap();

        for i in 0..10 {
            let entry = file_system.read_directory(directory).unwrap().unwrap();

            assert_eq!(*entry.get_name(), format!("Test{i}"));
            assert_eq!(entry.get_type(), Kind::File);
        }
    }

    pub async fn test_rewind_directory(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        // Create multiple files
        for i in 0..10 {
            let flags = Flags::new(Mode::WRITE_ONLY, Some(Open::CREATE_ONLY), None);
            let file = file_system
                .open(
                    task,
                    Path::from_str(&format!("/Test{i}")),
                    flags,
                    Time::new(123),
                    UserIdentifier::ROOT,
                    GroupIdentifier::ROOT,
                )
                .unwrap();
            file_system.close(file).unwrap();
        }

        let directory = file_system.open_directory(Path::ROOT, task).unwrap();

        let current_directory = file_system.read_directory(directory).unwrap().unwrap();
        assert_eq!(*current_directory.get_name(), ".");
        assert_eq!(current_directory.get_type(), Kind::Directory);

        let parent_directory = file_system.read_directory(directory).unwrap().unwrap();
        assert_eq!(*parent_directory.get_name(), "..");
        assert_eq!(parent_directory.get_type(), Kind::Directory);

        for i in 0..10 {
            let entry = file_system.read_directory(directory).unwrap().unwrap();

            assert_eq!(*entry.get_name(), format!("Test{i}"));
            assert_eq!(entry.get_type(), Kind::File);
        }

        file_system.rewind_directory(directory).unwrap();

        let current_directory = file_system.read_directory(directory).unwrap().unwrap();
        assert_eq!(*current_directory.get_name(), ".");
        assert_eq!(current_directory.get_type(), Kind::Directory);

        let parent_directory = file_system.read_directory(directory).unwrap().unwrap();
        assert_eq!(*parent_directory.get_name(), "..");
        assert_eq!(parent_directory.get_type(), Kind::Directory);

        for i in 0..10 {
            let entry = file_system.read_directory(directory).unwrap().unwrap();

            assert_eq!(*entry.get_name(), format!("Test{i}"));
            assert_eq!(entry.get_type(), Kind::File);
        }

        file_system.close_directory(directory).unwrap();
    }

    pub async fn test_create_remove_directory(file_system: impl FileSystemTraits) {
        let task = task::get_instance().get_current_task_identifier().await;

        let path = get_test_path().append("Test_create_directory").unwrap();

        file_system
            .create_directory(
                &path,
                Time::new(123),
                UserIdentifier::ROOT,
                GroupIdentifier::ROOT,
            )
            .unwrap();

        {
            let root_directory = file_system.open_directory(Path::ROOT, task).unwrap();

            let current_directory = file_system.read_directory(root_directory).unwrap().unwrap();
            assert_eq!(*current_directory.get_name(), ".");
            assert_eq!(current_directory.get_type(), Kind::Directory);

            let parent_directory = file_system.read_directory(root_directory).unwrap().unwrap();
            assert_eq!(*parent_directory.get_name(), "..");
            assert_eq!(parent_directory.get_type(), Kind::Directory);

            let directory = file_system.read_directory(root_directory).unwrap().unwrap();
            assert_eq!(*directory.get_name(), "Test_create_directory");
            assert_eq!(directory.get_type(), Kind::Directory);

            file_system.close_directory(root_directory).unwrap();
        }

        {
            let directory = file_system.open_directory(&path, task).unwrap();

            let current_directory = file_system.read_directory(directory).unwrap().unwrap();

            assert_eq!(*current_directory.get_name(), ".");
            assert_eq!(current_directory.get_type(), Kind::Directory);

            let parent_directory = file_system.read_directory(directory).unwrap().unwrap();
            assert_eq!(*parent_directory.get_name(), "..");
            assert_eq!(parent_directory.get_type(), Kind::Directory);

            file_system.close_directory(directory).unwrap();
        }
        file_system.remove(&path).unwrap();
    }
}
