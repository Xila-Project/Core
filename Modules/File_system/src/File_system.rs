use std::collections::BTreeMap;

use crate::{Inode_type, Local_file_identifier_type, Mode_type, Statistics_type};

use super::{
    Error_type, Flags_type, Path_type, Permissions_type, Position_type, Result_type, Size_type,
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
    ) -> Result_type<Local_file_identifier_type>;

    /// Close a file.
    ///
    /// # Errors
    /// Returns an error if the file is not opened by the task (invalid file identifier).
    /// Returns an error if the task identifier is invalid.
    fn Close(&self, File: Local_file_identifier_type) -> Result_type<()>;

    /// Close all files opened by the task.
    fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()>;

    /// Duplicate a file identifier.
    fn Duplicate_file_identifier(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type>;

    /// Transfer a file identifier from a task to another.
    fn Transfert_file_identifier(
        &self,
        New_task: Task_identifier_type,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type>;

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
    fn Read(&self, File: Local_file_identifier_type, Buffer: &mut [u8]) -> Result_type<Size_type>;

    /// Write a file.
    ///
    /// # Errors
    /// - If the file is not opened (invalid file identifier).
    /// - If the file is not opened in write mode (invalid mode).
    fn Write(&self, File: Local_file_identifier_type, Buffer: &[u8]) -> Result_type<Size_type>;

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
        File: Local_file_identifier_type,
        Position: &Position_type,
    ) -> Result_type<Size_type>;

    fn Flush(&self, File: Local_file_identifier_type) -> Result_type<()>;

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
        Ok(())
    }

    /// Set the permissions of the file.
    ///
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to set the permissions (no execute permission on parent directory).
    fn Set_permissions(&self, _: &dyn AsRef<Path_type>, _: Permissions_type) -> Result_type<()> {
        Ok(())
    }

    // - Directory

    fn Get_statistics(&self, File: Local_file_identifier_type) -> Result_type<Statistics_type>;

    fn Get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type>;
}

pub fn Get_new_file_identifier<T>(
    Task_identifier: Task::Task_identifier_type,
    Map: &BTreeMap<Local_file_identifier_type, T>,
) -> Result_type<Local_file_identifier_type> {
    let Iterator = Local_file_identifier_type::Get_minimum(Task_identifier);

    for Identifier in Iterator {
        if !Map.contains_key(&Identifier) {
            return Ok(Identifier);
        }
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

#[cfg(test)]
pub mod Tests {
    use Users::Root_user_identifier;

    use crate::{Open_type, Path_owned_type};

    use super::*;

    pub fn Get_test_path() -> Path_owned_type {
        Path_type::Get_root().to_owned()
    }

    pub fn Initialize() -> Task_identifier_type {
        let _ = Users::Initialize();

        if let Err(Error) = Task::Initialize() {
            unsafe {
                Task::Get_instance().Register_task();
            }
        }

        let _ = Time::Initialize(Box::new(Drivers::Native::Time_driver_type::New()));

        Task::Get_instance()
            .Get_current_task_identifier()
            .expect("Error getting current task identifier")
    }

    pub fn Test_open_close_delete(File_system: impl File_system_traits) {
        let Task = Initialize();

        let Path = Get_test_path().Append("Test_open_close_delete").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Delete(&Path).unwrap();
    }

    pub fn Test_read_write(File_system: impl File_system_traits) {
        let Task = Initialize();

        let Path = Get_test_path().Append("Test_read_write").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system.Write(File, &Buffer).unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));
        File_system.Set_position(File, &Position_type::Start(0)).unwrap();

        // - Read
        let mut Buffer_read = [0; 3];
        let Size = File_system.Read(File, &mut Buffer_read).unwrap();
        assert_eq!(Buffer, Buffer_read);
        assert_eq!(Size, Size_type::from(Buffer.len()));

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Delete(&Path).unwrap();
    }

    pub fn Test_move(File_system: impl File_system_traits) {
        let Task = Initialize();

        let Path = Get_test_path().Append("Test_move").unwrap();
        let Path_destination = Get_test_path().Append("Test_move_destination").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system.Write(File, &Buffer).unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));

        File_system.Close(File).unwrap();

        // - Move
        File_system.Move(&Path, &Path_destination).unwrap();

        // - Open
        let File = File_system
            .Open(Task, &Path_destination, Mode_type::Read_only.into())
            .unwrap();

        // - Read
        let mut Buffer_read = [0; 3];
        let Size = File_system.Read(File, &mut Buffer_read).unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));
        assert_eq!(Buffer, Buffer_read);

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Delete(&Path_destination).unwrap();
    }

    pub fn Test_set_position(File_system: impl File_system_traits) {
        let Task = Initialize();

        let Path = Get_test_path().Append("Test_set_position").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system.Write(File, &Buffer).unwrap();
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
        let Size = File_system.Read(File, &mut Buffer_read).unwrap();
        assert_eq!(Buffer, Buffer_read);
        assert_eq!(Buffer.len(), Size.into());

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Delete(&Path).unwrap();
    }

    pub fn Test_flush(File_system: impl File_system_traits) {
        let Task = Initialize();

        let Path = Get_test_path().Append("Test_flush").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system.Write(File, &Buffer).unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));

        // - Flush
        File_system.Flush(File).unwrap();

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Delete(&Path).unwrap();
    }

    pub fn Test_set_owner(File_system: impl File_system_traits) {
        let Task = Initialize();

        let Path = Get_test_path().Append("Test_set_owner").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Set owner
        File_system
            .Set_owner(&Path, Some(Root_user_identifier), None)
            .unwrap();

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Delete(&Path).unwrap();
    }

    pub fn Test_set_permissions(File_system: impl File_system_traits) {
        let Task = Initialize();

        let Path = Get_test_path().Append("Test_set_permissions").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Set permissions
        File_system
            .Set_permissions(&Path, Permissions_type::All_read_write)
            .unwrap();

        // - Close
        File_system.Close(File).unwrap();

        // - Delete
        File_system.Delete(&Path).unwrap();
    }
}
