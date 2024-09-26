use std::mem::size_of;

use crate::{File_identifier_inner_type, File_system_identifier_type, Mode_type, Statistics_type};

use super::{
    Error_type, File_identifier_type, Flags_type, Path_owned_type, Path_type, Permissions_type,
    Position_type, Result_type, Size_type, Status_type,
};

use Task::{Task_identifier_inner_type, Task_identifier_type};
use Users::{Group_identifier_type, User_identifier_type};

/// File system trait.
///
/// This allows to abstract the file system implementation.
/// The file system implementation should be registered in `Virtual_file_system_type`.
/// The management of concurrent access to the file system is delegated to the implementation.
/// Thus, implementation should use a `RwLock` or `Mutex` to manage concurrency.
pub trait File_system_traits: Send + Sync {
    // - Status
    // - Manipulation
    // - - Open/close/delete

    /// Open a file.
    ///     
    /// # Arguments
    ///
    /// - `Task` : Task identifier, used to identify the task since the file identifier is unique to the task.
    /// - `Path` : Path to the file.
    /// - `Flags` : Flags to open the file.
    /// - `File_identifier` : Optional file identifier, if not provided, a new file identifier is generated, otherwise, the provided file identifier is used.
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to open the file (mode is not compatible with the file permissions).
    /// Return an error if the provided file identifier is already used by the task.
    fn Open(
        &self,
        Task: Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<File_identifier_type>;

    /// Close a file.
    ///
    /// # Errors
    /// Returns an error if the file is not opened by the task (invalid file identifier).
    /// Returns an error if the task identifier is invalid.
    fn Close(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()>;

    /// Close all files opened by the task.
    fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()>;

    /// Duplicate a file identifier.
    fn Duplicate_file_identifier(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<File_identifier_type>;

    /// Transfer a file identifier from a task to another.
    fn Transfert_file_identifier(
        &self,
        Old_task: Task_identifier_type,
        New_task: Task_identifier_type,
        File: File_identifier_type,
        New_file_identifier: Option<File_identifier_type>,
    ) -> Result_type<File_identifier_type>;

    /// Delete a file.
    ///
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to delete the file (no write permission on parent directory).
    fn Delete(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()>;
    // - - File operations

    /// Read a file.
    ///
    /// # Errors
    /// - If the file is not opened.
    /// - If the file is not opened in read mode.
    fn Read(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type>;

    /// Write a file.
    ///
    /// # Errors
    /// - If the file is not opened (invalid file identifier).
    /// - If the file is not opened in write mode (invalid mode).
    fn Write(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<Size_type>;

    fn Move(
        &self,
        Source: &dyn AsRef<Path_type>,
        Destination: &dyn AsRef<Path_type>,
    ) -> Result_type<()>;

    /// Set the position of the file.
    ///
    /// # Errors
    /// - If the file is not opened (invalid file identifier).
    fn Set_position(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Position: &Position_type,
    ) -> Result_type<Size_type>;

    fn Flush(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()>;

    // - Metadata
    // - - Size

    // - - Security

    /// Set the owner of the file.
    /// If `User` is `None`, the owner is not changed.
    /// If `Group` is `None`, the group is not changed.
    /// If both are `None`, the owner and group are not changed.
    ///
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to change the owner (not the current owner or not the root user).
    fn Set_owner(
        &self,
        _: &dyn AsRef<Path_type>,
        _: Option<User_identifier_type>,
        _: Option<Group_identifier_type>,
    ) -> Result_type<()> {
        Ok(()) // TODO : Implement with permission file
    }

    /// Set the permissions of the file.
    ///
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to set the permissions (no execute permission on parent directory).
    fn Set_permissions(&self, _: &dyn AsRef<Path_type>, _: Permissions_type) -> Result_type<()> {
        Ok(()) // TODO : Implement with permission file
    }

    // - Directory

    fn Create_named_pipe(
        &self,
        _: &dyn AsRef<Path_type>,
        _: Size_type,
        _: User_identifier_type,
        _: Group_identifier_type,
        _: Permissions_type,
    ) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    //    fn Add_device(
    //        &self,
    //        _: &'static dyn AsRef<Path_type>,
    //        _: Box<dyn Device_trait>,
    //    ) -> Result_type<()> {
    //        Err(Error_type::Unsupported_operation)
    //    }

    fn Create_unnamed_pipe(
        &self,
        _: Task_identifier_type,
        _: Size_type,
        _: Status_type,
        _: User_identifier_type,
        _: Group_identifier_type,
        _: Permissions_type,
    ) -> Result_type<(File_identifier_type, File_identifier_type)> {
        Err(Error_type::Unsupported_operation)
    }

    fn Get_statistics(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        File_system: File_system_identifier_type,
    ) -> Result_type<Statistics_type>;

    fn Get_mode(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<Mode_type>;

    /// Combine task identifier and file identifier to get a unique file identifier.
    fn Get_local_file_identifier(
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
    ) -> usize
    where
        Self: Sized, // ? : Makes the compiler happy
    {
        let File_identifier: File_identifier_inner_type = File_identifier.into();
        let Task_identifier: Task_identifier_inner_type = Task_identifier.into();

        (Task_identifier as usize) << (size_of::<File_identifier_type>() * 8)
            | (File_identifier as usize)
    }

    fn Decompose_local_file_identifier(
        Local_file_identifier: usize,
    ) -> (Task_identifier_type, File_identifier_type)
    where
        Self: Sized, // ? : Makes the compiler happy
    {
        let Task_identifier = Local_file_identifier >> File_identifier_inner_type::BITS;
        let Task_identifier_type =
            Task_identifier_type::from(Task_identifier as Task_identifier_inner_type);

        let File_identifier =
            File_identifier_type::from(Local_file_identifier as File_identifier_inner_type);

        (Task_identifier_type, File_identifier)
    }

    // - Tests

    /// Test opening and closing a file.
    ///
    /// # Before running the tests
    ///
    /// - Create file `read_only`, `write_only` and `read_write` in the directory
    /// - Ensure `not_exists` doesn't exists in the `Test_path` directory
    /// - Ensure `read_only`, `write_only` and `read_write` are closed
    fn Test_open_close_file(&self) {
        let Task_identifier = Task_identifier_type::from(1);

        let Read_only = self
            .Open(
                Task_identifier,
                &Get_test_path().Append("read_only").unwrap(),
                Mode_type::Read_only.into(),
            )
            .unwrap();
        assert!(self
            .Open(
                Task_identifier,
                &Get_test_path().Append("read_only").unwrap(),
                Mode_type::Read_only.into(),
            )
            .is_err());

        let Write_only = self
            .Open(
                Task_identifier,
                &Get_test_path().Append("write_only").unwrap(),
                Mode_type::Write_only.into(),
            )
            .unwrap();

        let Read_write = self
            .Open(
                Task_identifier,
                &Get_test_path().Append("read_write").unwrap(),
                Mode_type::Read_write.into(),
            )
            .unwrap();

        self.Close(Task_identifier, Read_only).unwrap();

        self.Close(Task_identifier, Write_only).unwrap();

        self.Close(Task_identifier, Read_write).unwrap();
    }

    /// Test read file operation.
    ///
    /// # Before running the tests
    ///
    /// - Create file `read` in the `Test_path` directory containing `0123456789\n` (10 bytes)
    /// - Create file `empty_read` in the `Test_path` directory
    fn Test_file_read(&self) {
        let Task_identifier = Task_identifier_type::from(1);

        let Read_file = Get_test_path().Append("read").unwrap();
        let Read_file_identifier = self
            .Open(Task_identifier, &Read_file, Mode_type::Read_only.into())
            .unwrap();
        let mut Buffer = [0; 11];
        let Size = self
            .Read(Task_identifier, Read_file_identifier, &mut Buffer)
            .unwrap();
        assert_eq!(Size, 11);
        assert_eq!(&Buffer, b"0123456789\n");

        let Empty_file = Get_test_path().Append("empty_read").unwrap();
        let Empty_file_identifier = self
            .Open(Task_identifier, &Empty_file, Mode_type::Read_only.into())
            .unwrap();

        let mut Buffer = [0; 1];
        let Size = self
            .Read(Task_identifier, Empty_file_identifier, &mut Buffer)
            .unwrap();
        assert_eq!(Size, 0);
    }

    /// Test write file operation.
    ///
    /// # Before running the tests
    ///
    /// - Create file `write` in the `Test_path` directory
    fn Test_file_write(&self) {
        let Task_identifier = Task_identifier_type::from(1);

        let File = Get_test_path().Append("write").unwrap();
        let File_identifier = self
            .Open(Task_identifier, &File, Mode_type::Write_only.into())
            .unwrap();
        let Buffer = b"0123456789\n";
        let Size = self
            .Write(Task_identifier, File_identifier, Buffer)
            .unwrap();
        assert_eq!(Size, 11);
    }
}

pub fn Get_test_path() -> Path_owned_type {
    Path_type::Get_root().Append("test").unwrap()
}
