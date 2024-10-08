use std::collections::BTreeMap;

use crate::{
    Entry_type, File_identifier_type, Inode_type, Local_file_identifier_type, Metadata_type,
    Mode_type, Statistics_type,
};

use super::{Error_type, Flags_type, Path_type, Position_type, Result_type, Size_type};

use Task::Task_identifier_type;

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
    fn Read(&self, File: Local_file_identifier_type, Buffer: &mut [u8]) -> Result_type<Size_type>;

    /// Write a file.
    ///
    /// # Errors
    /// - If the file is not opened (invalid file identifier).
    /// - If the file is not opened in write mode (invalid mode).
    fn Write(&self, File: Local_file_identifier_type, Buffer: &[u8]) -> Result_type<Size_type>;

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

    fn Create_directory(&self, Path: &Path_type, Task: Task_identifier_type) -> Result_type<()>;

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

pub mod Tests {
    use std::sync::RwLock;

    use crate::{Device_trait, Open_type, Path_owned_type, Type_type};

    use super::*;

    pub struct Memory_device_type<const Block_size: usize>(RwLock<(Vec<u8>, usize)>);

    impl<const Block_size: usize> Memory_device_type<Block_size> {
        pub fn New(Size: usize) -> Self {
            assert!(Size % Block_size == 0);

            let Data: Vec<u8> = vec![0; Size];

            Self(RwLock::new((Data, 0)))
        }

        pub fn Get_block_count(&self) -> usize {
            self.0.read().unwrap().0.len() / Block_size
        }
    }

    impl<const Block_size: usize> Device_trait for Memory_device_type<Block_size> {
        fn Read(&self, Buffer: &mut [u8]) -> crate::Result_type<Size_type> {
            let mut Inner = self
                .0
                .try_write()
                .map_err(|_| crate::Error_type::Ressource_busy)?;
            let (Data, Position) = &mut *Inner;

            let Read_size = Buffer.len().min(Data.len().saturating_sub(*Position));
            Buffer[..Read_size].copy_from_slice(&Data[*Position..*Position + Read_size]);
            *Position += Read_size;
            Ok(Read_size.into())
        }

        fn Write(&self, Buffer: &[u8]) -> crate::Result_type<Size_type> {
            let mut Inner = self
                .0
                .write()
                .map_err(|_| crate::Error_type::Ressource_busy)?;
            let (Data, Position) = &mut *Inner;

            Data[*Position..*Position + Buffer.len()].copy_from_slice(Buffer);
            *Position += Buffer.len();
            Ok(Buffer.len().into())
        }

        fn Get_size(&self) -> crate::Result_type<Size_type> {
            let Inner = self
                .0
                .read()
                .map_err(|_| crate::Error_type::Ressource_busy)?;
            Ok(Size_type::New(Inner.0.len() as u64))
        }

        fn Set_position(&self, Position: &Position_type) -> crate::Result_type<Size_type> {
            let mut Inner = self
                .0
                .write()
                .map_err(|_| crate::Error_type::Ressource_busy)?;
            let (Data, Device_position) = &mut *Inner;

            match Position {
                Position_type::Start(Position) => *Device_position = *Position as usize,
                Position_type::Current(Position) => {
                    *Device_position = (*Device_position as isize + *Position as isize) as usize
                }
                Position_type::End(Position) => {
                    *Device_position = (Data.len() as isize - *Position as isize) as usize
                }
            }

            Ok(Size_type::New(*Device_position as u64))
        }

        fn Erase(&self) -> crate::Result_type<()> {
            let mut Inner = self
                .0
                .write()
                .map_err(|_| crate::Error_type::Ressource_busy)?;

            let (Data, Position) = &mut *Inner;

            Data[*Position..*Position + Block_size].fill(0);

            Ok(())
        }

        fn Flush(&self) -> crate::Result_type<()> {
            Ok(())
        }

        fn Get_block_size(&self) -> crate::Result_type<usize> {
            Ok(Block_size)
        }
    }

    pub fn Get_test_path() -> Path_owned_type {
        Path_type::Root.to_owned()
    }

    pub fn Test_open_close_delete(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

        let Path = Get_test_path().Append("Test_open_close_delete").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        // - Open
        let File = File_system.Open(Task, &Path, Flags).unwrap();

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
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system.Write(File, &Buffer).unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));
        File_system
            .Set_position(File, &Position_type::Start(0))
            .unwrap();

        // - Read
        let mut Buffer_read = [0; 3];
        let Size = File_system.Read(File, &mut Buffer_read).unwrap();
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
        let File = File_system.Open(Task, &Path, Flags).unwrap();

        // - Write
        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system.Write(File, &Buffer).unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));

        File_system.Close(File).unwrap();

        // - Move
        File_system.Rename(&Path, &Path_destination).unwrap();

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
        File_system.Remove(&Path_destination).unwrap();
    }

    pub fn Test_set_position(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

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
        File_system.Remove(&Path).unwrap();
    }

    pub fn Test_flush(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

        let Path = Get_test_path().Append("Test_flush").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        let File = File_system.Open(Task, &Path, Flags).unwrap();

        let Buffer = [0x01, 0x02, 0x03];
        let Size = File_system.Write(File, &Buffer).unwrap();
        assert_eq!(Size, Size_type::from(Buffer.len()));

        File_system.Flush(File).unwrap();

        File_system.Close(File).unwrap();

        File_system.Remove(&Path).unwrap();
    }

    pub fn Test_set_get_metadata(File_system: impl File_system_traits) {
        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

        let Path = Get_test_path().Append("Test_set_owner").unwrap();

        let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None);

        let File = File_system.Open(Task, &Path, Flags).unwrap();

        let Metadata = Metadata_type::Get_default(Task, Type_type::File).unwrap();

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
                .Open(Task, Path_type::From_str(&format!("/Test{}", i)), Flags)
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
                .Open(Task, Path_type::From_str(&format!("/Test{}", i)), Flags)
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
                .Open(Task, Path_type::From_str(&format!("/Test{}", i)), Flags)
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

        File_system.Create_directory(&Path, Task).unwrap();

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
