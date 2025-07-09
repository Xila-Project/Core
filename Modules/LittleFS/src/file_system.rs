use core::mem::MaybeUninit;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, ffi::CString, vec::Vec};
use file_system::{
    get_new_file_identifier, Device_type, Entry_type, File_identifier_inner_type,
    File_identifier_type, File_system_identifier_type, File_system_traits, Flags_type, Inode_type,
    Local_file_identifier_type, Metadata_type, Mode_type, Path_type, Permissions_type,
    Position_type, Size_type, Statistics_type, Time_type, Type_type,
};
use futures::block_on;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use users::{Group_identifier_type, User_identifier_type};

use super::{convert_result, littlefs, Configuration_type, Directory_type, File_type};

use file_system::{Error_type, Result_type};

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
        let mut inner = self.write_inner();

        let keys = inner.open_files.keys().cloned().collect::<Vec<_>>();

        for key in keys {
            if let Some(file) = inner.open_files.remove(&key) {
                let _ = file.close(&mut inner.file_system);
            }
        }

        let configuration =
            unsafe { Box::from_raw(inner.file_system.cfg as *mut littlefs::lfs_config) };

        let _read_buffer = unsafe {
            Vec::from_raw_parts(
                configuration.read_buffer as *mut u8,
                0,
                configuration.cache_size as usize,
            )
        };
        let _write_buffer = unsafe {
            Vec::from_raw_parts(
                configuration.prog_buffer as *mut u8,
                0,
                configuration.cache_size as usize,
            )
        };
        let _look_ahead_buffer = unsafe {
            Vec::from_raw_parts(
                configuration.lookahead_buffer as *mut u8,
                0,
                configuration.lookahead_size as usize,
            )
        };

        // Get the device
        let _device =
            unsafe { Box::from_raw(inner.file_system.cfg.read().context as *mut Device_type) };

        // - Unmount the file system
        unsafe {
            littlefs::lfs_unmount(&mut inner.file_system as *mut _);
        }

        // Configuration, Buffer sand Device are dropped here
    }
}

impl File_system_type {
    pub fn new(device: Device_type, cache_size: usize) -> Result_type<Self> {
        let block_size = device
            .get_block_size()
            .map_err(|_| Error_type::Input_output)?;
        let size = device.get_size().map_err(|_| Error_type::Input_output)?;

        let configuration: littlefs::lfs_config = Configuration_type::new(
            device,
            block_size,
            usize::from(size),
            cache_size,
            cache_size,
        )
        .ok_or(Error_type::Invalid_parameter)?
        .try_into()
        .map_err(|_| Error_type::Invalid_parameter)?;

        let configuration = Box::new(configuration);

        let mut file_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        convert_result(unsafe {
            littlefs::lfs_mount(
                file_system.as_mut_ptr() as *mut _,
                Box::into_raw(configuration),
            )
        })?;

        let inner = Inner_type {
            file_system: unsafe { file_system.assume_init() },
            open_files: BTreeMap::new(),
            open_directories: BTreeMap::new(),
        };

        Ok(Self {
            inner: RwLock::new(inner),
            cache_size,
        })
    }

    pub fn format(Device: Device_type, Cache_size: usize) -> Result_type<()> {
        let block_size = Device
            .get_block_size()
            .map_err(|_| Error_type::Input_output)?;
        let size = Device.get_size().map_err(|_| Error_type::Input_output)?;

        let configuration: littlefs::lfs_config = Configuration_type::new(
            Device,
            block_size,
            usize::from(size),
            Cache_size,
            Cache_size,
        )
        .ok_or(Error_type::Invalid_parameter)?
        .try_into()
        .map_err(|_| Error_type::Invalid_parameter)?;

        let configuration = Box::new(configuration);

        let mut file_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        convert_result(unsafe {
            littlefs::lfs_format(file_system.as_mut_ptr(), Box::into_raw(configuration))
        })?;

        Ok(())
    }

    fn borrow_mutable_inner_2_splitted(
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

    pub fn is_file(File: Local_file_identifier_type) -> bool {
        File.Split().1 < Self::DIRECTORY_MINIMUM
    }

    fn read_inner(
        &self,
    ) -> synchronization::rwlock::RwLockReadGuard<'_, CriticalSectionRawMutex, Inner_type> {
        block_on(self.inner.read())
    }

    fn write_inner(
        &self,
    ) -> synchronization::rwlock::RwLockWriteGuard<'_, CriticalSectionRawMutex, Inner_type> {
        block_on(self.inner.write())
    }
}

unsafe impl Send for File_system_type {}

unsafe impl Sync for File_system_type {}

impl File_system_traits for File_system_type {
    fn open(
        &self,
        task: task::Task_identifier_type,
        path: &Path_type,
        flags: Flags_type,
        time: Time_type,
        user: User_identifier_type,
        group: Group_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.write_inner();

        let file = File_type::open(
            &mut inner.file_system,
            path,
            flags,
            self.cache_size,
            time,
            user,
            group,
        )?;

        let file_identifier = get_new_file_identifier(
            task,
            Some(File_identifier_type::MINIMUM),
            Some(Self::DIRECTORY_MINIMUM),
            &inner.open_files,
        )?;

        if inner.open_files.insert(file_identifier, file).is_some() {
            return Err(Error_type::Internal_error);
        }

        Ok(file_identifier)
    }

    fn close(&self, file: Local_file_identifier_type) -> Result_type<()> {
        let mut inner = self.write_inner();

        let file = inner
            .open_files
            .remove(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        file.close(&mut inner.file_system)?;

        Ok(())
    }

    fn close_all(&self, Task: task::Task_identifier_type) -> Result_type<()> {
        let mut inner = self.write_inner();

        // Get all the keys of the open files that belong to the task
        let keys = inner
            .open_files
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the files corresponding to the keys
        for key in keys {
            if let Some(file) = inner.open_files.remove(&key) {
                file.close(&mut inner.file_system)?;
            }
        }

        Ok(())
    }

    fn duplicate(
        &self,
        file: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let (task, _) = file.Split();

        let mut inner = self.write_inner();

        let file = inner
            .open_files
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        let file = file.clone();

        let file_identifier = get_new_file_identifier(
            task,
            Some(Self::DIRECTORY_MINIMUM),
            Some(File_identifier_type::MAXIMUM),
            &inner.open_files,
        )?;

        if inner.open_files.insert(file_identifier, file).is_some() {
            return Err(Error_type::Internal_error);
        }

        Ok(file_identifier)
    }

    fn transfert(
        &self,
        new_task: task::Task_identifier_type,
        file_identifier: Local_file_identifier_type,
        new_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.write_inner();

        let file = inner
            .open_files
            .remove(&file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        let file_identifier = if let Some(new_file) = new_file {
            let file = Local_file_identifier_type::New(new_task, new_file);

            if inner.open_files.contains_key(&file) {
                return Err(Error_type::Invalid_identifier);
            }

            file
        } else if Self::is_file(file_identifier) {
            get_new_file_identifier(
                new_task,
                Some(File_identifier_type::MINIMUM),
                Some(Self::DIRECTORY_MINIMUM),
                &inner.open_files,
            )?
        } else {
            get_new_file_identifier(
                new_task,
                Some(Self::DIRECTORY_MINIMUM),
                Some(File_identifier_type::MAXIMUM),
                &inner.open_directories,
            )?
        };

        if inner.open_files.insert(file_identifier, file).is_some() {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(file_identifier)
    }

    fn remove(&self, Path: &Path_type) -> Result_type<()> {
        let path = CString::new(Path.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut inner = self.write_inner();

        convert_result(unsafe {
            littlefs::lfs_remove(&mut inner.file_system as *mut _, path.as_ptr())
        })?;

        Ok(())
    }

    fn read(
        &self,
        file: Local_file_identifier_type,
        buffer: &mut [u8],
        _: Time_type,
    ) -> Result_type<Size_type> {
        let mut inner = self.write_inner();

        let (file_system, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        file.read(file_system, buffer)
    }

    fn write(
        &self,
        file: Local_file_identifier_type,
        buffer: &[u8],
        _: Time_type,
    ) -> Result_type<Size_type> {
        let mut inner = self.write_inner();

        let (file_system, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        file.write(file_system, buffer)
    }

    fn rename(&self, Source: &Path_type, Destination: &Path_type) -> Result_type<()> {
        let source = CString::new(Source.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let destination =
            CString::new(Destination.As_str()).map_err(|_| Error_type::Invalid_parameter)?;

        let mut inner = self.write_inner();

        convert_result(unsafe {
            littlefs::lfs_rename(
                &mut inner.file_system as *mut _,
                source.as_ptr(),
                destination.as_ptr(),
            )
        })?;

        Ok(())
    }

    fn set_position(
        &self,
        file: Local_file_identifier_type,
        position: &Position_type,
    ) -> Result_type<Size_type> {
        let mut inner = self.write_inner();

        let (file_system, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        file.set_position(file_system, position)
    }

    fn flush(&self, file: Local_file_identifier_type) -> Result_type<()> {
        let mut inner = self.write_inner();

        let (file_system, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        file.flush(file_system)
    }

    fn get_statistics(&self, File: Local_file_identifier_type) -> Result_type<Statistics_type> {
        let mut inner = self.write_inner();

        let (file_system, open_files, open_directories) =
            Self::borrow_mutable_inner_2_splitted(&mut inner);

        // open_directoriesy to get the metadata of the directories
        if open_directories.get_mut(&File).is_some() {
            let current_time: Time_type = time::get_instance().get_current_time().unwrap().into();

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
        } else if let Some(file) = open_files.get_mut(&File) {
            Ok(file.get_statistics(file_system)?)
        } else {
            Err(Error_type::Invalid_identifier)
        }
    }

    fn get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type> {
        let inner = self.read_inner();

        let result = if Self::is_file(File) {
            inner
                .open_files
                .get(&File)
                .ok_or(Error_type::Invalid_identifier)?
                .get_mode()
        } else {
            inner
                .open_directories
                .get(&File)
                .ok_or(Error_type::Invalid_identifier)?;

            Mode_type::READ_ONLY
        };

        Ok(result)
    }

    fn open_directory(
        &self,
        path: &Path_type,
        task: task::Task_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.write_inner();

        let directory = Directory_type::open(&mut inner.file_system, path)?;

        let file_identifier = get_new_file_identifier(
            task,
            Some(Self::DIRECTORY_MINIMUM),
            Some(File_identifier_type::MAXIMUM),
            &inner.open_directories,
        )?;

        if inner
            .open_directories
            .insert(file_identifier, directory)
            .is_some()
        {
            return Err(Error_type::Internal_error);
        }

        Ok(file_identifier)
    }

    fn read_directory(&self, File: Local_file_identifier_type) -> Result_type<Option<Entry_type>> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let directory = open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        directory.read(file_system)
    }

    fn set_position_directory(
        &self,
        file: Local_file_identifier_type,
        position: Size_type,
    ) -> Result_type<()> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let directory = open_directories
            .get_mut(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        directory.set_position(file_system, position)
    }

    fn rewind_directory(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let directory = open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        directory.rewind(file_system)?;

        Ok(())
    }

    fn close_directory(&self, File: Local_file_identifier_type) -> Result_type<()> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let mut directory = open_directories
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        directory.close(file_system)?;

        Ok(())
    }

    fn create_directory(
        &self,
        path: &Path_type,

        Time: Time_type,
        user: User_identifier_type,
        group: Group_identifier_type,
    ) -> Result_type<()> {
        let mut inner = self.write_inner();

        Directory_type::create_directory(&mut inner.file_system, path)?;

        let metadata = Metadata_type::get_default(Type_type::Directory, Time, user, group)
            .ok_or(Error_type::Invalid_parameter)?;

        File_type::set_metadata_from_path(&mut inner.file_system, path, &metadata)?;

        Ok(())
    }

    fn get_position_directory(&self, File: Local_file_identifier_type) -> Result_type<Size_type> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let directory = open_directories
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        directory.get_position(file_system)
    }

    fn set_metadata_from_path(
        &self,
        path: &Path_type,
        metadata: &Metadata_type,
    ) -> Result_type<()> {
        let mut inner = self.write_inner();

        File_type::set_metadata_from_path(&mut inner.file_system, path, metadata)?;

        Ok(())
    }

    fn get_metadata_from_path(&self, Path: &Path_type) -> Result_type<Metadata_type> {
        let mut inner = self.write_inner();

        File_type::get_metadata_from_path(&mut inner.file_system, Path)
    }

    fn get_metadata(&self, File: Local_file_identifier_type) -> Result_type<Metadata_type> {
        let mut inner = self.write_inner();

        let (_, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files
            .get_mut(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(file.get_metadata()?.clone())
    }
}

#[cfg(test)]
mod tests {

    use alloc::sync::Arc;
    use file_system::{Create_device, Memory_device_type};
    use task::Test;

    use super::*;

    const Cache_size: usize = 256;

    fn initialize() -> File_system_type {
        let _ = users::initialize();

        task::initialize();

        let _ = time::initialize(Create_device!(drivers::native::Time_driver_type::new()));

        let mock_device = Memory_device_type::<512>::New(2048 * 512);

        let device = Device_type::New(Arc::new(mock_device));

        File_system_type::format(device.clone(), Cache_size).unwrap();

        File_system_type::new(device, Cache_size).unwrap()
    }

    #[Test]
    async fn test_open_close_delete() {
        file_system::tests::Test_open_close_delete(initialize()).await;
    }

    #[Test]
    async fn test_read_write() {
        file_system::tests::Test_read_write(initialize()).await;
    }

    #[Test]
    async fn test_move() {
        file_system::tests::Test_move(initialize()).await;
    }

    #[Test]
    async fn test_set_position() {
        file_system::tests::Test_set_position(initialize()).await;
    }

    #[Test]
    async fn test_flush() {
        file_system::tests::Test_flush(initialize()).await;
    }

    #[Test]
    async fn test_set_get_metadata() {
        file_system::tests::Test_set_get_metadata(initialize()).await;
    }

    #[Test]
    async fn test_read_directory() {
        file_system::tests::Test_read_directory(initialize()).await;
    }

    #[Test]
    async fn test_set_position_directory() {
        file_system::tests::Test_set_position_directory(initialize()).await;
    }

    #[Test]
    async fn test_rewind_directory() {
        file_system::tests::Test_rewind_directory(initialize()).await;
    }

    #[Test]
    async fn test_create_remove_directory() {
        file_system::tests::Test_create_remove_directory(initialize()).await;
    }

    #[cfg(feature = "std")]
    #[Test]
    async fn test_loader() {
        file_system::tests::Test_loader(initialize());
    }
}
