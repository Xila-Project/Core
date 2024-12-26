use std::collections::BTreeMap;

use crate::{
    Entry_type, File_identifier_type, Inode_type, Local_file_identifier_type, Metadata_type,
    Mode_type, Statistics_type, Time_type,
};

use super::{Error_type, Flags_type, Path_type, Position_type, Result_type, Size_type};

use Task::Task_identifier_type;
use Users::{Group_identifier_type, User_identifier_type};

#[macro_export]
macro_rules! Create_file_system {
    ($file_system:expr) => {
        std::boxed::Box::new($file_system)
    };
    () => {};
}

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
        Path: &Path_type,
        Flags: Flags_type,
        Time: Time_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
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
    fn Duplicate(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type>;

    /// Transfer a file identifier from a task to another.
    fn Transfert(
        &self,
        New_task: Task_identifier_type,
        File: Local_file_identifier_type,
        New_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type>;

    /// Remove a file or a directory.
    ///
    /// # Errors
    /// Returns an error if the file doesn't exists.
    /// Returns an error if the user / group doesn't have the permission to delete the file (no write permission on parent directory).
    fn Remove(&self, Path: &Path_type) -> Result_type<()>;
    // - - File operations

    /// Read a file.
    ///
    /// # Errors
    /// - If the file is not opened.
    /// - If the file is not opened in read mode.
    fn Read(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut [u8],
        Time_type: Time_type,
    ) -> Result_type<Size_type>;

    fn Read_line(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut String,
        Time_type: Time_type,
    ) -> Result_type<Size_type> {
        loop {
            let Current_buffer = &mut [0; 1];

            let Size = self.Read(File, Current_buffer, Time_type)?;

            if Size == 0 {
                break;
            }

            let Byte = Current_buffer[0];

            if Byte == b'\n' || Byte == b'\r' {
                break;
            }

            Buffer.push(Byte as char);
        }

        Ok(Buffer.len().into())
    }

    /// Write a file.
    ///
    /// # Errors
    /// - If the file is not opened (invalid file identifier).
    /// - If the file is not opened in write mode (invalid mode).
    fn Write(
        &self,
        File: Local_file_identifier_type,
        Buffer: &[u8],
        Time_type: Time_type,
    ) -> Result_type<Size_type>;

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

    use crate::{Loader::Loader_type, Open_type, Path_owned_type, Time_type, Type_type};

    use super::*;

    pub fn Get_test_path() -> Path_owned_type {
        Path_type::Root.to_owned()
    }

    pub fn Test_open_close_delete(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

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

    pub fn Test_read_write(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

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

    pub fn Test_move(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

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

    pub fn Test_set_position(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

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

    pub fn Test_flush(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

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

    pub fn Test_set_get_metadata(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

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

    pub fn Test_read_directory(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

        // Create multiple files
        for i in 0..10 {
            let Flags = Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None);
            let File = File_system
                .Open(
                    Task,
                    Path_type::From_str(&format!("/Test{}", i)),
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

            assert_eq!(*Entry.Get_name(), format!("Test{}", i));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }

        File_system.Close_directory(Directory).unwrap();
    }

    pub fn Test_set_position_directory(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

        // Create multiple files
        for i in 0..10 {
            let Flags = Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None);
            let File = File_system
                .Open(
                    Task,
                    Path_type::From_str(&format!("/Test{}", i)),
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

            assert_eq!(*Entry.Get_name(), format!("Test{}", i));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }

        File_system
            .Set_position_directory(Directory, Position)
            .unwrap();

        for i in 0..10 {
            let Entry = File_system.Read_directory(Directory).unwrap().unwrap();

            assert_eq!(*Entry.Get_name(), format!("Test{}", i));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }
    }

    pub fn Test_rewind_directory(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

        // Create multiple files
        for i in 0..10 {
            let Flags = Flags_type::New(Mode_type::Write_only, Some(Open_type::Create_only), None);
            let File = File_system
                .Open(
                    Task,
                    Path_type::From_str(&format!("/Test{}", i)),
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

            assert_eq!(*Entry.Get_name(), format!("Test{}", i));
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

            assert_eq!(*Entry.Get_name(), format!("Test{}", i));
            assert_eq!(Entry.Get_type(), Type_type::File);
        }

        File_system.Close_directory(Directory).unwrap();
    }

    pub fn Test_create_remove_directory(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

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

    pub fn Test_loader(mut File_system: impl File_system_traits) {
        // - Load the file in the file system
        let Source_path = "Cargo.toml";
        let Destination_path = "/Cargo.toml";

        let Loader = Loader_type::New().Add_file(Source_path, Destination_path);

        Loader.Load(&mut File_system).unwrap();

        // - Read the file and compare it with the original
        let Test_file = std::fs::read_to_string(Source_path).unwrap();

        let mut Buffer = vec![0; Test_file.len()];

        let File = File_system
            .Open(
                Task_identifier_type::New(0),
                Path_type::New(Destination_path),
                Flags_type::New(Mode_type::Read_only, None, None),
                Time_type::New(0),
                User_identifier_type::Root,
                Group_identifier_type::Root,
            )
            .unwrap();

        let Read = File_system
            .Read(File, &mut Buffer, Time_type::New(0))
            .unwrap();

        assert_eq!(Read, Test_file.len());
        assert_eq!(Buffer, Test_file.as_bytes());
    }
}
