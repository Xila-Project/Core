use core::mem::MaybeUninit;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, ffi::CString, vec::Vec};
use File_system::{
    Device_type, Entry_type, File_identifier_inner_type, File_identifier_type,
    File_system_identifier_type, File_system_traits, Flags_type, Get_new_file_identifier,
    Inode_type, Local_file_identifier_type, Metadata_type, Mode_type, Path_type, Permissions_type,
    Position_type, Size_type, Statistics_type, Time_type, Type_type,
};
use Futures::block_on;
use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use Users::{Group_identifier_type, User_identifier_type};

use super::{littlefs, Configuration_type, Convert_result, Directory_type, File_type};

use File_system::{Error_type, Result_type};

struct Inner_type {
    file_system: littlefs::lfs_t,
    open_files: BTreeMap<Local_file_identifier_type, File_type>,
    open_directories: BTreeMap<Local_file_identifier_type, Directory_type>,
}

pub struct File_system_type {
    inner: RwLock<CriticalSectionRawMutex, Inner_type>,
    cache_size: usize,
}

impl Drop for File_system_type {
    fn drop(&mut self) {
        // - Close all the open files
        let mut Inner = self.Write_inner();

        let Keys = Inner.open_files.keys().cloned().collect::<Vec<_>>();

        for Key in Keys {
            if let Some(file) = Inner.open_files.remove(&Key) {
                let _ = file.Close(&mut Inner.file_system);
            }
        }

        let Configuration =
            unsafe { Box::from_raw(Inner.file_system.cfg as *mut littlefs::lfs_config) };

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
            unsafe { Box::from_raw(Inner.file_system.cfg.read().context as *mut Device_type) };

        // - Unmount the file system
        unsafe {
            littlefs::lfs_unmount(&mut Inner.file_system as *mut _);
        }

        // Configuration, Buffer sand Device are dropped here
    }
}

impl File_system_type {
    pub fn new(device: Device_type, Cache_size: usize) -> Result_type<Self> {
        let block_size = device
            .Get_block_size()
            .map_err(|_| Error_type::Input_output)?;
        let size = device.Get_size().map_err(|_| Error_type::Input_output)?;

        let Configuration: littlefs::lfs_config = Configuration_type::New(
            device,
            block_size,
            usize::from(size),
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
            file_system: unsafe { File_system.assume_init() },
            open_files: BTreeMap::new(),
            open_directories: BTreeMap::new(),
        };

        Ok(Self {
            inner: RwLock::new(Inner),
            cache_size: Cache_size,
        })
    }

    pub fn Format(Device: Device_type, Cache_size: usize) -> Result_type<()> {
        let block_size = Device
            .Get_block_size()
            .map_err(|_| Error_type::Input_output)?;
        let size = Device.Get_size().map_err(|_| Error_type::Input_output)?;

        let Configuration: littlefs::lfs_config = Configuration_type::New(
            Device,
            block_size,
            usize::from(size),
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
        inner_2: &mut Inner_type,
    ) -> (
        &mut littlefs::lfs_t,
        &mut BTreeMap<Local_file_identifier_type, File_type>,
        &mut BTreeMap<Local_file_identifier_type, Directory_type>,
    ) {
        (
            &mut inner_2.file_system,
            &mut inner_2.open_files,
            &mut inner_2.open_directories,
        )
    }

    #[cfg(target_pointer_width = "64")]
    const DIRECTORY_FLAG: File_identifier_inner_type = 1 << 31;
    #[cfg(target_pointer_width = "32")]
    const Directory_flag: File_identifier_inner_type = 1 << 15;

    const DIRECTORY_MINIMUM: File_identifier_type = File_identifier_type::New(Self::DIRECTORY_FLAG);

    pub fn Is_file(File: Local_file_identifier_type) -> bool {
        File.Split().1 < Self::DIRECTORY_MINIMUM
    }

    fn Read_inner(
        &self,
    ) -> Synchronization::rwlock::RwLockReadGuard<'_, CriticalSectionRawMutex, Inner_type> {
        block_on(self.inner.read())
    }

    fn Write_inner(
        &self,
    ) -> Synchronization::rwlock::RwLockWriteGuard<'_, CriticalSectionRawMutex, Inner_type> {
        block_on(self.inner.write())
    }
}

unsafe impl Send for File_system_type {}

unsafe impl Sync for File_system_type {}

impl File_system_traits for File_system_type {
    fn Open(
        &self,
        task: Task::Task_identifier_type,
        path: &Path_type,
        flags: Flags_type,
        time: Time_type,
        user: User_identifier_type,
        group: Group_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.Write_inner();

        let File = File_type::open(
            &mut inner.file_system,
            path,
            flags,
            self.cache_size,
            time,
            user,
            group,
        )?;

        let File_identifier = Get_new_file_identifier(
            task,
            Some(File_identifier_type::MINIMUM),
            Some(Self::DIRECTORY_MINIMUM),
            &inner.open_files,
        )?;

        if inner.open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error);
        }

        Ok(File_identifier)
    }

    fn Close(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut inner = self.Write_inner();

        let File = inner
            .open_files
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Close(&mut inner.file_system)?;

        Ok(())
    }

    fn Close_all(&self, Task: Task::Task_identifier_type) -> Result_type<()> {
        let mut inner = self.Write_inner();

        // Get all the keys of the open files that belong to the task
        let Keys = inner
            .open_files
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the files corresponding to the keys
        for Key in Keys {
            if let Some(file) = inner.open_files.remove(&Key) {
                file.Close(&mut inner.file_system)?;
            }
        }

        Ok(())
    }

    fn Duplicate(
        &self,
        file: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let (task, _) = file.Split();

        let mut Inner = self.Write_inner();

        let File = Inner
            .open_files
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        let File = File.clone();

        let File_identifier = Get_new_file_identifier(
            task,
            Some(Self::DIRECTORY_MINIMUM),
            Some(File_identifier_type::MAXIMUM),
            &Inner.open_files,
        )?;

        if Inner.open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error);
        }

        Ok(File_identifier)
    }

    fn Transfert(
        &self,
        new_task: Task::Task_identifier_type,
        file_identifier: Local_file_identifier_type,
        new_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.Write_inner();

        let File = inner
            .open_files
            .remove(&file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        let File_identifier = if let Some(New_file) = new_file {
            let file = Local_file_identifier_type::New(new_task, New_file);

            if inner.open_files.contains_key(&file) {
                return Err(Error_type::Invalid_identifier);
            }

            file
        } else if Self::Is_file(file_identifier) {
            Get_new_file_identifier(
                new_task,
                Some(File_identifier_type::MINIMUM),
                Some(Self::DIRECTORY_MINIMUM),
                &inner.open_files,
            )?
        } else {
            Get_new_file_identifier(
                new_task,
                Some(Self::DIRECTORY_MINIMUM),
                Some(File_identifier_type::MAXIMUM),
                &inner.open_directories,
            )?
        };

        if inner.open_files.insert(File_identifier, File).is_some() {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(File_identifier)
    }

    fn Remove(&self, Path: &Path_type) -> Result_type<()> {
        let path = CString::new(Path.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut Inner = self.Write_inner();

        Convert_result(unsafe {
            littlefs::lfs_remove(&mut Inner.file_system as *mut _, path.as_ptr())
        })?;

        Ok(())
    }

    fn Read(
        &self,
        file: Local_file_identifier_type,
        buffer: &mut [u8],
        _: Time_type,
    ) -> Result_type<Size_type> {
        let mut inner = self.Write_inner();

        let (File_system, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let File = Open_files
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Read(File_system, buffer)
    }

    fn Write(
        &self,
        file: Local_file_identifier_type,
        buffer: &[u8],
        _: Time_type,
    ) -> Result_type<Size_type> {
        let mut inner = self.Write_inner();

        let (File_system, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let File = Open_files
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Write(File_system, buffer)
    }

    fn Rename(&self, Source: &Path_type, Destination: &Path_type) -> Result_type<()> {
        let source = CString::new(Source.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let Destination =
            CString::new(Destination.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut Inner = self.Write_inner();

        Convert_result(unsafe {
            littlefs::lfs_rename(
                &mut Inner.file_system as *mut _,
                source.as_ptr(),
                Destination.as_ptr(),
            )
        })?;

        Ok(())
    }

    fn Set_position(
        &self,
        file: Local_file_identifier_type,
        position: &Position_type,
    ) -> Result_type<Size_type> {
        let mut inner = self.Write_inner();

        let (File_system, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let File = Open_files
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Set_position(File_system, position)
    }

    fn Flush(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut inner = self.Write_inner();

        let (File_system, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        File.Flush(File_system)
    }

    fn Get_statistics(&self, File: Local_file_identifier_type) -> Result_type<Statistics_type> {
        let mut inner = self.Write_inner();

        let (File_system, Open_files, Open_directories) =
            Self::Borrow_mutable_inner_2_splitted(&mut inner);

        // TODO : Find a way to get the metadata of the directories
        if Open_directories.get_mut(&File).is_some() {
            let current_time: Time_type = Time::Get_instance().Get_current_time().unwrap().into();

            Ok(Statistics_type::new(
                File_system_identifier_type::New(0),
                Inode_type::New(0),
                1,
                Size_type::New(0),
                current_time,
                current_time,
                current_time,
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
        let inner = self.Read_inner();

        let Result = if Self::Is_file(File) {
            inner
                .open_files
                .get(&File)
                .ok_or(Error_type::Invalid_identifier)?
                .Get_mode()
        } else {
            inner
                .open_directories
                .get(&File)
                .ok_or(Error_type::Invalid_identifier)?;

            Mode_type::READ_ONLY
        };

        Ok(Result)
    }

    fn Open_directory(
        &self,
        path: &Path_type,
        task: Task::Task_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.Write_inner();

        let Directory = Directory_type::Open(&mut inner.file_system, path)?;

        let File_identifier = Get_new_file_identifier(
            task,
            Some(Self::DIRECTORY_MINIMUM),
            Some(File_identifier_type::MAXIMUM),
            &inner.open_directories,
        )?;

        if inner
            .open_directories
            .insert(File_identifier, Directory)
            .is_some()
        {
            return Err(Error_type::Internal_error);
        }

        Ok(File_identifier)
    }

    fn Read_directory(&self, File: Local_file_identifier_type) -> Result_type<Option<Entry_type>> {
        let mut inner = self.Write_inner();

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let Directory = Open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Read(File_system)
    }

    fn Set_position_directory(
        &self,
        file: Local_file_identifier_type,
        position: Size_type,
    ) -> Result_type<()> {
        let mut inner = self.Write_inner();

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let Directory = Open_directories
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Set_position(File_system, position)
    }

    fn Rewind_directory(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut inner = self.Write_inner();

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let Directory = Open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Rewind(File_system)?;

        Ok(())
    }

    fn Close_directory(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut inner = self.Write_inner();

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let mut Directory = Open_directories
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Close(File_system)?;

        Ok(())
    }

    fn Create_directory(
        &self,
        path: &Path_type,

        Time: Time_type,
        user: User_identifier_type,
        group: Group_identifier_type,
    ) -> Result_type<()> {
        let mut inner = self.Write_inner();

        Directory_type::create_directory(&mut inner.file_system, path)?;

        let Metadata = Metadata_type::Get_default(Type_type::Directory, Time, user, group)
            .ok_or(Error_type::Invalid_parameter)?;

        File_type::Set_metadata_from_path(&mut inner.file_system, path, &Metadata)?;

        Ok(())
    }

    fn Get_position_directory(&self, File: Local_file_identifier_type) -> Result_type<Size_type> {
        let mut inner = self.Write_inner();

        let (File_system, _, Open_directories) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let Directory = Open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Directory.Get_position(File_system)
    }

    fn Set_metadata_from_path(
        &self,
        path: &Path_type,
        metadata: &Metadata_type,
    ) -> Result_type<()> {
        let mut inner = self.Write_inner();

        File_type::Set_metadata_from_path(&mut inner.file_system, path, metadata)?;

        Ok(())
    }

    fn Get_metadata_from_path(&self, Path: &Path_type) -> Result_type<Metadata_type> {
        let mut inner = self.Write_inner();

        File_type::Get_metadata_from_path(&mut inner.file_system, Path)
    }

    fn Get_metadata(&self, File: Local_file_identifier_type) -> Result_type<Metadata_type> {
        let mut inner = self.Write_inner();

        let (_, Open_files, _) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let File = Open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(File.Get_metadata()?.clone())
    }
}

#[cfg(test)]
mod tests {

    use alloc::sync::Arc;
    use File_system::{Create_device, Memory_device_type};
    use Task::Test;

    use super::*;

    const Cache_size: usize = 256;

    fn Initialize() -> File_system_type {
        let _ = Users::Initialize();

        Task::Initialize();

        let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

        let Mock_device = Memory_device_type::<512>::New(2048 * 512);

        let Device = Device_type::New(Arc::new(Mock_device));

        File_system_type::Format(Device.clone(), Cache_size).unwrap();

        File_system_type::new(Device, Cache_size).unwrap()
    }

    #[Test]
    async fn test_open_close_delete() {
        File_system::Tests::Test_open_close_delete(Initialize()).await;
    }

    #[Test]
    async fn test_read_write() {
        File_system::Tests::Test_read_write(Initialize()).await;
    }

    #[Test]
    async fn test_move() {
        File_system::Tests::Test_move(Initialize()).await;
    }

    #[Test]
    async fn test_set_position() {
        File_system::Tests::Test_set_position(Initialize()).await;
    }

    #[Test]
    async fn test_flush() {
        File_system::Tests::Test_flush(Initialize()).await;
    }

    #[Test]
    async fn test_set_get_metadata() {
        File_system::Tests::Test_set_get_metadata(Initialize()).await;
    }

    #[Test]
    async fn test_read_directory() {
        File_system::Tests::Test_read_directory(Initialize()).await;
    }

    #[Test]
    async fn test_set_position_directory() {
        File_system::Tests::Test_set_position_directory(Initialize()).await;
    }

    #[Test]
    async fn test_rewind_directory() {
        File_system::Tests::Test_rewind_directory(Initialize()).await;
    }

    #[Test]
    async fn test_create_remove_directory() {
        File_system::Tests::Test_create_remove_directory(Initialize()).await;
    }

    #[cfg(feature = "std")]
    #[Test]
    async fn test_loader() {
        File_system::Tests::Test_loader(Initialize());
    }
}
