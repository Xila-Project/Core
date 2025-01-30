use core::mem::MaybeUninit;
use std::{collections::BTreeMap, ffi::CString, sync::RwLock};

use File_system::{
    Device_type, Entry_type, File_identifier_inner_type, File_identifier_type,
    File_system_identifier_type, File_system_traits, Flags_type, Get_new_file_identifier,
    Inode_type, Local_file_identifier_type, Metadata_type, Mode_type, Path_type, Permissions_type,
    Position_type, Size_type, Statistics_type, Time_type, Type_type,
};
use Users::{Group_identifier_type, User_identifier_type};

use super::{littlefs, Configuration_type, Convert_result, Directory_type, File_type};

use File_system::{Error_type, Result_type};

struct Inner_type {
    File_system: littlefs::lfs_t,
    Open_files: BTreeMap<Local_file_identifier_type, File_type>,
    Open_directories: BTreeMap<Local_file_identifier_type, Directory_type>,
}

pub struct File_system_type {
    Inner: RwLock<Inner_type>,
    Cache_size: usize,
}

impl Drop for File_system_type {
    fn drop(&mut self) {
        // - Close all the open files
        let mut Inner = self.Inner.write().unwrap();

        let Keys = Inner.Open_files.keys().cloned().collect::<Vec<_>>();

        for Key in Keys {
            if let Some(File) = Inner.Open_files.remove(&Key) {
                let _ = File.Close(&mut Inner.File_system);
            }
        }

        let Configuration =
            unsafe { Box::from_raw(Inner.File_system.cfg as *mut littlefs::lfs_config) };

        let _Read_buffer = unsafe {
            Vec::from_raw_parts(
                Configuration.read_buffer as *mut u8,
                0,
                Configuration.cache_size as usize,
            )
        };
        let _Write_buffer = unsafe {
            Vec::from_raw_parts(
                Configuration.prog_buffer as *mut u8,
                0,
                Configuration.cache_size as usize,
            )
        };
        let _Look_ahead_buffer = unsafe {
            Vec::from_raw_parts(
                Configuration.lookahead_buffer as *mut u8,
                0,
                Configuration.lookahead_size as usize,
            )
        };

        // Get the device
        let _Device =
            unsafe { Box::from_raw(Inner.File_system.cfg.read().context as *mut Device_type) };

        // - Unmount the file system
        unsafe {
            littlefs::lfs_unmount(&mut Inner.File_system as *mut _);
        }

        // Configuration, Buffer sand Device are dropped here
    }
}

impl File_system_type {
    pub fn New(Device: Device_type, Cache_size: usize) -> Result_type<Self> {
        let Block_size = Device
            .Get_block_size()
            .map_err(|_| Error_type::Input_output)?;
        let Size = Device.Get_size().map_err(|_| Error_type::Input_output)?;

        let Configuration: littlefs::lfs_config = Configuration_type::New(
            Device,
            Block_size,
            usize::from(Size),
            Cache_size,
            Cache_size,
        )
        .ok_or(Error_type::Invalid_parameter)?
        .try_into()
        .map_err(|_| Error_type::Invalid_parameter)?;

        let Configuration = Box::new(Configuration);

        let mut File_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        Convert_result(unsafe {
            littlefs::lfs_mount(
                File_system.as_mut_ptr() as *mut _,
                Box::into_raw(Configuration),
            )
        })?;

        let Inner = Inner_type {
            File_system: unsafe { File_system.assume_init() },
            Open_files: BTreeMap::new(),
            Open_directories: BTreeMap::new(),
        };

        Ok(Self {
            Inner: RwLock::new(Inner),
            Cache_size,
        })
    }

    pub fn Format(Device: Device_type, Cache_size: usize) -> Result_type<()> {
        let Block_size = Device
            .Get_block_size()
            .map_err(|_| Error_type::Input_output)?;
        let Size = Device.Get_size().map_err(|_| Error_type::Input_output)?;

        let Configuration: littlefs::lfs_config = Configuration_type::New(
            Device,
            Block_size,
            usize::from(Size),
            Cache_size,
            Cache_size,
        )
        .ok_or(Error_type::Invalid_parameter)?
        .try_into()
        .map_err(|_| Error_type::Invalid_parameter)?;

        let Configuration = Box::new(Configuration);

        let mut File_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        Convert_result(unsafe {
            littlefs::lfs_format(File_system.as_mut_ptr(), Box::into_raw(Configuration))
        })?;

        Ok(())
    }

    fn Borrow_mutable_inner_2_splitted(
        Inner_2: &mut Inner_type,
    ) -> (
        &mut littlefs::lfs_t,
        &mut BTreeMap<Local_file_identifier_type, File_type>,
        &mut BTreeMap<Local_file_identifier_type, Directory_type>,
    ) {
        (
            &mut Inner_2.File_system,
            &mut Inner_2.Open_files,
            &mut Inner_2.Open_directories,
        )
    }

    #[cfg(target_pointer_width = "64")]
    const Directory_flag: File_identifier_inner_type = 1 << 31;
    #[cfg(target_pointer_width = "32")]
    const Directory_flag: File_identifier_inner_type = 1 << 15;

    const Directory_minimum: File_identifier_type = File_identifier_type::New(Self::Directory_flag);

    pub fn Is_file(File: Local_file_identifier_type) -> bool {
        File.Split().1 < Self::Directory_minimum
    }
}

unsafe impl Send for File_system_type {}

unsafe impl Sync for File_system_type {}

impl File_system_traits for File_system_type {
    fn Open(
        &self,
        Task: Task::Task_identifier_type,
        Path: &Path_type,
        Flags: Flags_type,
        Time: Time_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.Inner.write()?;

        let File = File_type::Open(
            &mut Inner.File_system,
            Path,
            Flags,
            self.Cache_size,
            Time,
            User,
            Group,
        )?;

        let File_identifier = Get_new_file_identifier(
            Task,
            Some(File_identifier_type::Minimum),
            Some(Self::Directory_minimum),
            &Inner.Open_files,
        )?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error);
        }

        Ok(File_identifier)
    }

    fn Close(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut Inner = self.Inner.write()?;

        let File = Inner
            .Open_files
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Close(&mut Inner.File_system)?;

        Ok(())
    }

    fn Close_all(&self, Task: Task::Task_identifier_type) -> Result_type<()> {
        let mut Inner = self.Inner.write()?;

        // Get all the keys of the open files that belong to the task
        let Keys = Inner
            .Open_files
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the files corresponding to the keys
        for Key in Keys {
            if let Some(File) = Inner.Open_files.remove(&Key) {
                File.Close(&mut Inner.File_system)?;
            }
        }

        Ok(())
    }

    fn Duplicate(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let (Task, _) = File.Split();

        let mut Inner = self.Inner.write()?;

        let File = Inner
            .Open_files
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let File = File.clone();

        let File_identifier = Get_new_file_identifier(
            Task,
            Some(Self::Directory_minimum),
            Some(File_identifier_type::Maximum),
            &Inner.Open_files,
        )?;

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error);
        }

        Ok(File_identifier)
    }

    fn Transfert(
        &self,
        New_task: Task::Task_identifier_type,
        File_identifier: Local_file_identifier_type,
        New_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.Inner.write()?;

        let File = Inner
            .Open_files
            .remove(&File_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        let File_identifier = if let Some(New_file) = New_file {
            let File = Local_file_identifier_type::New(New_task, New_file);

            if Inner.Open_files.contains_key(&File) {
                return Err(Error_type::Invalid_identifier);
            }

            File
        } else if Self::Is_file(File_identifier) {
            Get_new_file_identifier(
                New_task,
                Some(File_identifier_type::Minimum),
                Some(Self::Directory_minimum),
                &Inner.Open_files,
            )?
        } else {
            Get_new_file_identifier(
                New_task,
                Some(Self::Directory_minimum),
                Some(File_identifier_type::Maximum),
                &Inner.Open_directories,
            )?
        };

        if Inner.Open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(File_identifier)
    }

    fn Remove(&self, Path: &Path_type) -> Result_type<()> {
        let Path = CString::new(Path.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut Inner = self.Inner.write()?;

        Convert_result(unsafe {
            littlefs::lfs_remove(&mut Inner.File_system as *mut _, Path.as_ptr())
        })?;

        Ok(())
    }

    fn Read(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut [u8],
        _: Time_type,
    ) -> Result_type<Size_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Read(File_system, Buffer)
    }

    fn Write(
        &self,
        File: Local_file_identifier_type,
        Buffer: &[u8],
        _: Time_type,
    ) -> Result_type<Size_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Write(File_system, Buffer)
    }

    fn Rename(&self, Source: &Path_type, Destination: &Path_type) -> Result_type<()> {
        let Source = CString::new(Source.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let Destination =
            CString::new(Destination.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut Inner = self.Inner.write()?;

        Convert_result(unsafe {
            littlefs::lfs_rename(
                &mut Inner.File_system as *mut _,
                Source.as_ptr(),
                Destination.as_ptr(),
            )
        })?;

        Ok(())
    }

    fn Set_position(
        &self,
        File: Local_file_identifier_type,
        Position: &Position_type,
    ) -> Result_type<Size_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Set_position(File_system, Position)
    }

    fn Flush(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Flush(File_system)
    }

    fn Get_statistics(&self, File: Local_file_identifier_type) -> Result_type<Statistics_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, Open_files, Open_directories) =
            Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        // TODO : Find a way to get the metadata of the directories
        if Open_directories.get_mut(&File).is_some() {
            let Current_time: Time_type = Time::Get_instance().Get_current_time().unwrap().into();

            Ok(Statistics_type::New(
                File_system_identifier_type::New(0),
                Inode_type::New(0),
                1,
                Size_type::New(0),
                Current_time,
                Current_time,
                Current_time,
                Type_type::Directory,
                Permissions_type::New_default(Type_type::Directory),
                User_identifier_type::New(0),
                Group_identifier_type::New(0),
            ))
        } else if let Some(File) = Open_files.get_mut(&File) {
            Ok(File.Get_statistics(File_system)?)
        } else {
            Err(Error_type::Invalid_identifier)
        }
    }

    fn Get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type> {
        let Inner = self.Inner.read()?;

        let Result = if Self::Is_file(File) {
            Inner
                .Open_files
                .get(&File)
                .ok_or(Error_type::Invalid_identifier)?
                .Get_mode()
        } else {
            Inner
                .Open_directories
                .get(&File)
                .ok_or(Error_type::Invalid_identifier)?;

            Mode_type::Read_only
        };

        Ok(Result)
    }

    fn Open_directory(
        &self,
        Path: &Path_type,
        Task: Task::Task_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.Inner.write()?;

        let Directory = Directory_type::Open(&mut Inner.File_system, Path)?;

        let File_identifier = Get_new_file_identifier(
            Task,
            Some(Self::Directory_minimum),
            Some(File_identifier_type::Maximum),
            &Inner.Open_directories,
        )?;

        if Inner
            .Open_directories
            .insert(File_identifier, Directory)
            .is_some()
        {
            return Err(Error_type::Internal_error);
        }

        Ok(File_identifier)
    }

    fn Read_directory(&self, File: Local_file_identifier_type) -> Result_type<Option<Entry_type>> {
        let mut Inner = self.Inner.write()?;

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let Directory = Open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Read(File_system)
    }

    fn Set_position_directory(
        &self,
        File: Local_file_identifier_type,
        Position: Size_type,
    ) -> Result_type<()> {
        let mut Inner = self.Inner.write()?;

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let Directory = Open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Set_position(File_system, Position)
    }

    fn Rewind_directory(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut Inner = self.Inner.write()?;

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let Directory = Open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Rewind(File_system)?;

        Ok(())
    }

    fn Close_directory(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut Inner = self.Inner.write()?;

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let mut Directory = Open_directories
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Close(File_system)?;

        Ok(())
    }

    fn Create_directory(
        &self,
        Path: &Path_type,

        Time: Time_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
    ) -> Result_type<()> {
        let mut Inner = self.Inner.write()?;

        Directory_type::Create_directory(&mut Inner.File_system, Path)?;

        let Metadata = Metadata_type::Get_default(Type_type::Directory, Time, User, Group)
            .ok_or(Error_type::Invalid_parameter)?;

        File_type::Set_metadata_from_path(&mut Inner.File_system, Path, &Metadata)?;

        Ok(())
    }

    fn Get_position_directory(&self, File: Local_file_identifier_type) -> Result_type<Size_type> {
        let mut Inner = self.Inner.write()?;

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let Directory = Open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Get_position(File_system)
    }

    fn Set_metadata_from_path(
        &self,
        Path: &Path_type,
        Metadata: &Metadata_type,
    ) -> Result_type<()> {
        let mut Inner = self.Inner.write()?;

        File_type::Set_metadata_from_path(&mut Inner.File_system, Path, Metadata)?;

        Ok(())
    }

    fn Get_metadata_from_path(&self, Path: &Path_type) -> Result_type<Metadata_type> {
        let mut Inner = self.Inner.write()?;

        File_type::Get_metadata_from_path(&mut Inner.File_system, Path)
    }

    fn Get_metadata(&self, File: Local_file_identifier_type) -> Result_type<Metadata_type> {
        let mut Inner = self.Inner.write()?;

        let (_, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Get_metadata()?.clone())
    }
}

#[cfg(test)]
mod Tests {

    use std::sync::Arc;

    use File_system::{Create_device, Memory_device_type};

    use super::*;

    const Cache_size: usize = 256;

    fn Initialize() -> File_system_type {
        let _ = Users::Initialize();

        Task::Initialize().unwrap();

        unsafe {
            let _ = Task::Get_instance().Register_task();
        }

        let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

        let Mock_device = Memory_device_type::<512>::New(2048 * 512);

        let Device = Device_type::New(Arc::new(Mock_device));

        File_system_type::Format(Device.clone(), Cache_size).unwrap();

        File_system_type::New(Device, Cache_size).unwrap()
    }

    #[test]
    fn Test_open_close_delete() {
        File_system::Tests::Test_open_close_delete(Initialize());
    }

    #[test]
    fn Test_read_write() {
        File_system::Tests::Test_read_write(Initialize());
    }

    #[test]
    fn Test_move() {
        File_system::Tests::Test_move(Initialize());
    }

    #[test]
    fn Test_set_position() {
        File_system::Tests::Test_set_position(Initialize());
    }

    #[test]
    fn Test_flush() {
        File_system::Tests::Test_flush(Initialize());
    }

    #[test]
    fn Test_set_get_metadata() {
        File_system::Tests::Test_set_get_metadata(Initialize());
    }

    #[test]
    fn Test_read_directory() {
        File_system::Tests::Test_read_directory(Initialize());
    }

    #[test]
    fn Test_set_position_directory() {
        File_system::Tests::Test_set_position_directory(Initialize());
    }

    #[test]
    fn Test_rewind_directory() {
        File_system::Tests::Test_rewind_directory(Initialize());
    }

    #[test]
    fn Test_create_remove_directory() {
        File_system::Tests::Test_create_remove_directory(Initialize());
    }

    #[test]
    fn Test_loader() {
        File_system::Tests::Test_loader(Initialize());
    }
}
