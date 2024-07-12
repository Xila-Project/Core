use crate::Mode_type;

use super::{
    Device::Device_trait, Error_type, File_identifier_type, Flags_type, Path_owned_type, Path_type,
    Permissions_type, Position_type, Result_type, Size_type, Status_type, Type_type,
};

use Task::Task_identifier_type;
use Users::{Group_identifier_type, User_identifier_type};

/// File system trait.
///
/// This allows to abstract the file system implementation.
/// The file system implementation should be registered in `Virtual_file_system_type`.
/// The management of concurrent access to the file system is delegated to the implementation.
/// Thus, implementation should use a `RwLock` or `Mutex` to manage concurrency.
pub trait File_system_traits: Send + Sync {
    // - Status
    fn Exists(&self, Path: &dyn AsRef<Path_type>) -> Result_type<bool>;

    // - Manipulation
    // - - Open/close/delete

    /// Create a file.
    ///
    /// # Errors
    /// Returns an error if the file already exists.
    /// Returns an error if the user / group doesn't have the permission to create the file (no write permission on parent directory).
    fn Create_file(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()>;

    /// Open a file.
    ///     
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to open the file (mode is not compatible with the file permissions).
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

    /// Transfer a file identifier from a task to another.
    fn Transfert_file_identifier(
        &self,
        Old_task: Task_identifier_type,
        New_task: Task_identifier_type,
        File: File_identifier_type,
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

    /// Get the type of the file.
    ///
    /// # Errors
    /// - If the file doesn't exists.
    fn Get_type(&self, Path_type: &dyn AsRef<Path_type>) -> Result_type<Type_type>;

    /// Get the size of the file.
    ///
    /// # Errors
    /// - If the file doesn't exists.
    /// - If the user / group doesn't have the permission to get the size (no execute permission on parent directory).
    fn Get_size(&self, Path: &dyn AsRef<Path_type>) -> Result_type<Size_type>;

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

    /// Get the owner of the file.
    ///     
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to get the owner (no execute permission on parent directory).
    fn Get_owner(
        &self,
        _: &dyn AsRef<Path_type>,
    ) -> Result_type<(User_identifier_type, Group_identifier_type)> {
        Ok((0, 0)) // TODO : Implement with permission file
    }

    /// Set the permissions of the file.
    ///
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to set the permissions (no execute permission on parent directory).
    fn Set_permissions(&self, _: &dyn AsRef<Path_type>, _: Permissions_type) -> Result_type<()> {
        Ok(()) // TODO : Implement with permission file
    }

    /// Get the permissions of the file.
    ///
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to get the permissions (no execute permission on parent directory).
    fn Get_permissions(&self, _: &dyn AsRef<Path_type>) -> Result_type<Permissions_type> {
        Ok(Permissions_type::New_all_full()) // TODO : Implement with permission file
    }

    // - Directory

    /// Create a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory / file already exists.
    /// Returns an error if the user / group doesn't have the permission to create the directory (no write permission on parent directory).
    fn Create_directory(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()>;

    fn Create_named_pipe(&self, _: &dyn AsRef<Path_type>, _: Size_type) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    fn Add_device(
        &self,
        _: &'static dyn AsRef<Path_type>,
        _: Box<dyn Device_trait>,
    ) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    fn Create_unnamed_pipe(
        &self,
        _: Task_identifier_type,
        _: Size_type,
        _: Status_type,
    ) -> Result_type<(File_identifier_type, File_identifier_type)> {
        Err(Error_type::Unsupported_operation)
    }

    /// Combine task identifier and file identifier to get a unique file identifier.
    fn Get_local_file_identifier(
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
    ) -> u32
    where
        Self: Sized, // ? : Makes the compiler happy
    {
        let File_identifier: u16 = File_identifier.into();
        let Task_identifier: u32 = Task_identifier.into();
        (Task_identifier) << 16 | File_identifier as u32
    }

    fn Decompose_local_file_identifier(
        Local_file_identifier: u32,
    ) -> (Task_identifier_type, File_identifier_type)
    where
        Self: Sized, // ? : Makes the compiler happy
    {
        let Task_identifier = Task_identifier_type::from(Local_file_identifier >> 16);
        let File_identifier = File_identifier_type::from((Local_file_identifier & 0xFFFF) as u16);
        (Task_identifier, File_identifier)
    }

    // - Tests

    /// Test the existence of a file.
    ///
    /// # Before running the tests
    ///
    /// - `Test_path` should be an existing directory
    /// - Create file `exists` in the `Test_path` directory
    /// - Ensure `not_exists` doesn't exists in the `Test_path` directory
    fn Test_existence(&self) {
        assert_eq!(self.Exists(&Get_test_path()), Ok(true));
        assert_eq!(
            self.Exists(&Get_test_path().Append("exists").unwrap()),
            Ok(true)
        );
        assert_eq!(
            self.Exists(&Get_test_path().Append("not_exists").unwrap()),
            Ok(false)
        );
    }

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
                Mode_type::Read_only().into(),
            )
            .unwrap();
        assert!(self
            .Open(
                Task_identifier,
                &Get_test_path().Append("read_only").unwrap(),
                Mode_type::Read_only().into(),
            )
            .is_err());

        let Write_only = self
            .Open(
                Task_identifier,
                &Get_test_path().Append("write_only").unwrap(),
                Mode_type::Write_only().into(),
            )
            .unwrap();

        let Read_write = self
            .Open(
                Task_identifier,
                &Get_test_path().Append("read_write").unwrap(),
                Mode_type::Read_write().into(),
            )
            .unwrap();

        self.Close(Task_identifier, Read_only).unwrap();

        self.Close(Task_identifier, Write_only).unwrap();

        self.Close(Task_identifier, Read_write).unwrap();
    }

    /// Test creating a directory and verifying its existence.
    ///
    /// # Before running the tests
    ///
    /// - Ensure `test_dir` doesn't exists in the `Test_path` directory
    /// - Ensure `already_exists` exists in the `Test_path` directory
    fn Test_create_directory_exists(&self) {
        let New_path = Get_test_path().Append("test_dir").unwrap();

        assert_eq!(self.Exists(&New_path), Ok(false));
        self.Create_directory(&New_path).unwrap();
        assert_eq!(self.Exists(&New_path), Ok(true));
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
            .Open(Task_identifier, &Read_file, Mode_type::Read_only().into())
            .unwrap();
        let mut Buffer = [0; 11];
        let Size = self
            .Read(Task_identifier, Read_file_identifier, &mut Buffer)
            .unwrap();
        assert_eq!(Size, 11);
        assert_eq!(&Buffer, b"0123456789\n");
        assert_eq!(self.Get_size(&Read_file).unwrap(), 11);

        let Empty_file = Get_test_path().Append("empty_read").unwrap();
        let Empty_file_identifier = self
            .Open(Task_identifier, &Empty_file, Mode_type::Read_only().into())
            .unwrap();

        let mut Buffer = [0; 1];
        let Size = self
            .Read(Task_identifier, Empty_file_identifier, &mut Buffer)
            .unwrap();
        assert_eq!(Size, 0);
        assert_eq!(self.Get_size(&Empty_file).unwrap(), 0);
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
            .Open(Task_identifier, &File, Mode_type::Write_only().into())
            .unwrap();
        let Buffer = b"0123456789\n";
        let Size = self
            .Write(Task_identifier, File_identifier, Buffer)
            .unwrap();
        assert_eq!(Size, 11);
        assert_eq!(self.Get_size(&File).unwrap(), 11);
    }

    /// Run before the tests.
    fn Reset_test_directory(&self) {
        let _ = self.Delete(&Get_test_path());
        assert_eq!(self.Exists(&Get_test_path()), Ok(false));

        self.Create_directory(&Get_test_path()).unwrap();
        assert_eq!(self.Exists(&Get_test_path()), Ok(true));
    }
}

pub fn Get_test_path() -> Path_owned_type {
    Path_type::Get_root().Append("test").unwrap()
}
