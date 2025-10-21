use core::mem::MaybeUninit;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, ffi::CString, vec::Vec};
use file_system::{
    Device, Entry, FileIdentifier, FileIdentifierInner, FileSystemIdentifier, FileSystemTraits,
    Flags, Inode, Kind, LocalFileIdentifier, Metadata, Mode, Path, Permissions, Position, Size,
    Statistics_type, Time, get_new_file_identifier,
};
use futures::block_on;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use users::{GroupIdentifier, UserIdentifier};

use super::{Configuration, Directory, File, convert_result, littlefs};

use file_system::{Error, Result};

struct Inner {
    file_system: littlefs::lfs_t,
    open_files: BTreeMap<LocalFileIdentifier, File>,
    open_directories: BTreeMap<LocalFileIdentifier, Directory>,
}

pub struct FileSystem {
    inner: RwLock<CriticalSectionRawMutex, Inner>,
    cache_size: usize,
}

impl Drop for FileSystem {
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
        let _device = unsafe { Box::from_raw(inner.file_system.cfg.read().context as *mut Device) };

        // - Unmount the file system
        unsafe {
            littlefs::lfs_unmount(&mut inner.file_system as *mut _);
        }

        // Configuration, Buffer sand Device are dropped here
    }
}

impl FileSystem {
    pub fn new(device: Device, cache_size: usize) -> Result<Self> {
        let block_size = device.get_block_size().map_err(|_| Error::InputOutput)?;
        let size = device.get_size().map_err(|_| Error::InputOutput)?;

        let configuration: littlefs::lfs_config = Configuration::new(
            device,
            block_size,
            usize::from(size),
            cache_size,
            cache_size,
        )
        .ok_or(Error::InvalidParameter)?
        .try_into()
        .map_err(|_| Error::InvalidParameter)?;

        let configuration = Box::new(configuration);

        let mut file_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        convert_result(unsafe {
            littlefs::lfs_mount(
                file_system.as_mut_ptr() as *mut _,
                Box::into_raw(configuration),
            )
        })?;

        let inner = Inner {
            file_system: unsafe { file_system.assume_init() },
            open_files: BTreeMap::new(),
            open_directories: BTreeMap::new(),
        };

        Ok(Self {
            inner: RwLock::new(inner),
            cache_size,
        })
    }

    pub fn format(device: Device, cache_size: usize) -> Result<()> {
        let block_size = device.get_block_size().map_err(|_| Error::InputOutput)?;
        let size = device.get_size().map_err(|_| Error::InputOutput)?;

        let configuration: littlefs::lfs_config = Configuration::new(
            device,
            block_size,
            usize::from(size),
            cache_size,
            cache_size,
        )
        .ok_or(Error::InvalidParameter)?
        .try_into()
        .map_err(|_| Error::InvalidParameter)?;

        let configuration = Box::new(configuration);

        let mut file_system = MaybeUninit::<littlefs::lfs_t>::uninit();

        convert_result(unsafe {
            littlefs::lfs_format(file_system.as_mut_ptr(), Box::into_raw(configuration))
        })?;

        Ok(())
    }

    fn borrow_mutable_inner_2_splitted(
        inner_2: &mut Inner,
    ) -> (
        &mut littlefs::lfs_t,
        &mut BTreeMap<LocalFileIdentifier, File>,
        &mut BTreeMap<LocalFileIdentifier, Directory>,
    ) {
        (
            &mut inner_2.file_system,
            &mut inner_2.open_files,
            &mut inner_2.open_directories,
        )
    }

    #[cfg(target_pointer_width = "64")]
    const DIRECTORY_FLAG: FileIdentifierInner = 1 << 31;
    #[cfg(target_pointer_width = "32")]
    const DIRECTORY_FLAG: FileIdentifierInner = 1 << 15;

    const DIRECTORY_MINIMUM: FileIdentifier = FileIdentifier::new(Self::DIRECTORY_FLAG);

    pub fn is_file(file: LocalFileIdentifier) -> bool {
        file.split().1 < Self::DIRECTORY_MINIMUM
    }

    fn read_inner(
        &self,
    ) -> synchronization::rwlock::RwLockReadGuard<'_, CriticalSectionRawMutex, Inner> {
        block_on(self.inner.read())
    }

    fn write_inner(
        &self,
    ) -> synchronization::rwlock::RwLockWriteGuard<'_, CriticalSectionRawMutex, Inner> {
        block_on(self.inner.write())
    }
}

unsafe impl Send for FileSystem {}

unsafe impl Sync for FileSystem {}

impl FileSystemTraits for FileSystem {
    fn open(
        &self,
        task: task::TaskIdentifier,
        path: &Path,
        flags: Flags,
        time: Time,
        user: UserIdentifier,
        group: GroupIdentifier,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.write_inner();

        let file = File::open(
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
            Some(FileIdentifier::MINIMUM),
            Some(Self::DIRECTORY_MINIMUM),
            &inner.open_files,
        )?;

        if inner.open_files.insert(file_identifier, file).is_some() {
            return Err(Error::InternalError);
        }

        Ok(file_identifier)
    }

    fn close(&self, file: LocalFileIdentifier) -> Result<()> {
        let mut inner = self.write_inner();

        let file = inner
            .open_files
            .remove(&file)
            .ok_or(Error::InvalidIdentifier)?;

        file.close(&mut inner.file_system)?;

        Ok(())
    }

    fn close_all(&self, task: task::TaskIdentifier) -> Result<()> {
        let mut inner = self.write_inner();

        // Get all the keys of the open files that belong to the task
        let keys = inner
            .open_files
            .keys()
            .filter(|key| key.split().0 == task)
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

    fn duplicate(&self, file: LocalFileIdentifier) -> Result<LocalFileIdentifier> {
        let (task, _) = file.split();

        let mut inner = self.write_inner();

        let file = inner
            .open_files
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?;

        let file = file.clone();

        let file_identifier = get_new_file_identifier(
            task,
            Some(Self::DIRECTORY_MINIMUM),
            Some(FileIdentifier::MAXIMUM),
            &inner.open_files,
        )?;

        if inner.open_files.insert(file_identifier, file).is_some() {
            return Err(Error::InternalError);
        }

        Ok(file_identifier)
    }

    fn transfert(
        &self,
        new_task: task::TaskIdentifier,
        file_identifier: LocalFileIdentifier,
        new_file: Option<FileIdentifier>,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.write_inner();

        let file = inner
            .open_files
            .remove(&file_identifier)
            .ok_or(Error::InvalidIdentifier)?;

        let file_identifier = if let Some(new_file) = new_file {
            let file = LocalFileIdentifier::new(new_task, new_file);

            if inner.open_files.contains_key(&file) {
                return Err(Error::InvalidIdentifier);
            }

            file
        } else if Self::is_file(file_identifier) {
            get_new_file_identifier(
                new_task,
                Some(FileIdentifier::MINIMUM),
                Some(Self::DIRECTORY_MINIMUM),
                &inner.open_files,
            )?
        } else {
            get_new_file_identifier(
                new_task,
                Some(Self::DIRECTORY_MINIMUM),
                Some(FileIdentifier::MAXIMUM),
                &inner.open_directories,
            )?
        };

        if inner.open_files.insert(file_identifier, file).is_some() {
            return Err(Error::InternalError); // Should never happen
        }

        Ok(file_identifier)
    }

    fn remove(&self, path: &Path) -> Result<()> {
        let path = CString::new(path.as_str()).map_err(|_| Error::InvalidParameter)?;

        let mut inner = self.write_inner();

        convert_result(unsafe {
            littlefs::lfs_remove(&mut inner.file_system as *mut _, path.as_ptr())
        })?;

        Ok(())
    }

    fn read(&self, file: LocalFileIdentifier, buffer: &mut [u8], _: Time) -> Result<Size> {
        let mut inner = self.write_inner();

        let (file_system, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files.get_mut(&file).ok_or(Error::InvalidIdentifier)?;

        file.read(file_system, buffer)
    }

    fn write(&self, file: LocalFileIdentifier, buffer: &[u8], _: Time) -> Result<Size> {
        let mut inner = self.write_inner();

        let (file_system, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files.get_mut(&file).ok_or(Error::InvalidIdentifier)?;

        file.write(file_system, buffer)
    }

    fn rename(&self, source: &Path, destination: &Path) -> Result<()> {
        let source = CString::new(source.as_str()).map_err(|_| Error::InvalidParameter)?;

        let destination =
            CString::new(destination.as_str()).map_err(|_| Error::InvalidParameter)?;

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

    fn set_position(&self, file: LocalFileIdentifier, position: &Position) -> Result<Size> {
        let mut inner = self.write_inner();

        let (file_system, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files.get_mut(&file).ok_or(Error::InvalidIdentifier)?;

        file.set_position(file_system, position)
    }

    fn flush(&self, file: LocalFileIdentifier) -> Result<()> {
        let mut inner = self.write_inner();

        let (file_system, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files.get_mut(&file).ok_or(Error::InvalidIdentifier)?;

        file.flush(file_system)
    }

    fn get_statistics(&self, file: LocalFileIdentifier) -> Result<Statistics_type> {
        let mut inner = self.write_inner();

        let (file_system, open_files, open_directories) =
            Self::borrow_mutable_inner_2_splitted(&mut inner);

        // open_directoriesy to get the metadata of the directories
        if open_directories.get_mut(&file).is_some() {
            let current_time: Time = time::get_instance().get_current_time().unwrap().into();

            Ok(Statistics_type::new(
                FileSystemIdentifier::new(0),
                Inode::new(0),
                1,
                Size::new(0),
                current_time,
                current_time,
                current_time,
                Kind::Directory,
                Permissions::new_default(Kind::Directory),
                UserIdentifier::new(0),
                GroupIdentifier::new(0),
            ))
        } else if let Some(file) = open_files.get_mut(&file) {
            Ok(file.get_statistics(file_system)?)
        } else {
            Err(Error::InvalidIdentifier)
        }
    }

    fn get_mode(&self, file: LocalFileIdentifier) -> Result<Mode> {
        let inner = self.read_inner();

        let result = if Self::is_file(file) {
            inner
                .open_files
                .get(&file)
                .ok_or(Error::InvalidIdentifier)?
                .get_mode()
        } else {
            inner
                .open_directories
                .get(&file)
                .ok_or(Error::InvalidIdentifier)?;

            Mode::READ_ONLY
        };

        Ok(result)
    }

    fn open_directory(
        &self,
        path: &Path,
        task: task::TaskIdentifier,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.write_inner();

        let directory = Directory::open(&mut inner.file_system, path)?;

        let file_identifier = get_new_file_identifier(
            task,
            Some(Self::DIRECTORY_MINIMUM),
            Some(FileIdentifier::MAXIMUM),
            &inner.open_directories,
        )?;

        if inner
            .open_directories
            .insert(file_identifier, directory)
            .is_some()
        {
            return Err(Error::InternalError);
        }

        Ok(file_identifier)
    }

    fn read_directory(&self, file: LocalFileIdentifier) -> Result<Option<Entry>> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let directory = open_directories
            .get_mut(&file)
            .ok_or(Error::InvalidIdentifier)?;

        directory.read(file_system)
    }

    fn set_position_directory(&self, file: LocalFileIdentifier, position: Size) -> Result<()> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let directory = open_directories
            .get_mut(&file)
            .ok_or(Error::InvalidIdentifier)?;

        directory.set_position(file_system, position)
    }

    fn rewind_directory(&self, file: LocalFileIdentifier) -> Result<()> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let directory = open_directories
            .get_mut(&file)
            .ok_or(Error::InvalidIdentifier)?;

        directory.rewind(file_system)?;

        Ok(())
    }

    fn close_directory(&self, file: LocalFileIdentifier) -> Result<()> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let mut directory = open_directories
            .remove(&file)
            .ok_or(Error::InvalidIdentifier)?;

        directory.close(file_system)?;

        Ok(())
    }

    fn create_directory(
        &self,
        path: &Path,

        time: Time,
        user: UserIdentifier,
        group: GroupIdentifier,
    ) -> Result<()> {
        let mut inner = self.write_inner();

        Directory::create_directory(&mut inner.file_system, path)?;

        let metadata = Metadata::get_default(Kind::Directory, time, user, group)
            .ok_or(Error::InvalidParameter)?;

        File::set_metadata_from_path(&mut inner.file_system, path, &metadata)?;

        Ok(())
    }

    fn get_position_directory(&self, file: LocalFileIdentifier) -> Result<Size> {
        let mut inner = self.write_inner();

        let (file_system, _, open_directories) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let directory = open_directories
            .get_mut(&file)
            .ok_or(Error::InvalidIdentifier)?;

        directory.get_position(file_system)
    }

    fn set_metadata_from_path(&self, path: &Path, metadata: &Metadata) -> Result<()> {
        let mut inner = self.write_inner();

        File::set_metadata_from_path(&mut inner.file_system, path, metadata)?;

        Ok(())
    }

    fn get_metadata_from_path(&self, path: &Path) -> Result<Metadata> {
        let mut inner = self.write_inner();

        File::get_metadata_from_path(&mut inner.file_system, path)
    }

    fn get_metadata(&self, file: LocalFileIdentifier) -> Result<Metadata> {
        let mut inner = self.write_inner();

        let (_, open_files, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let file = open_files.get_mut(&file).ok_or(Error::InvalidIdentifier)?;

        Ok(file.get_metadata()?.clone())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::sync::Arc;
    use file_system::{MemoryDevice, create_device};
    use task::test;

    use super::*;

    const CACHE_SIZE: usize = 256;

    fn initialize() -> FileSystem {
        let _ = users::initialize();

        task::initialize();

        let _ = time::initialize(create_device!(drivers::native::TimeDriver::new()));

        let mock_device = MemoryDevice::<512>::new(2048 * 512);

        let device = Device::new(Arc::new(mock_device));

        FileSystem::format(device.clone(), CACHE_SIZE).unwrap();

        FileSystem::new(device, CACHE_SIZE).unwrap()
    }

    #[test]
    async fn test_open_close_delete() {
        file_system::tests::test_open_close_delete(initialize()).await;
    }

    #[test]
    async fn test_read_write() {
        file_system::tests::test_read_write(initialize()).await;
    }

    #[test]
    async fn test_move() {
        file_system::tests::test_move(initialize()).await;
    }

    #[test]
    async fn test_set_position() {
        file_system::tests::test_set_position(initialize()).await;
    }

    #[test]
    async fn test_flush() {
        file_system::tests::test_flush(initialize()).await;
    }

    #[test]
    async fn test_set_get_metadata() {
        file_system::tests::test_set_get_metadata(initialize()).await;
    }

    #[test]
    async fn test_read_directory() {
        file_system::tests::test_read_directory(initialize()).await;
    }

    #[test]
    async fn test_set_position_directory() {
        file_system::tests::test_set_position_directory(initialize()).await;
    }

    #[test]
    async fn test_rewind_directory() {
        file_system::tests::test_rewind_directory(initialize()).await;
    }

    #[test]
    async fn test_create_remove_directory() {
        file_system::tests::test_create_remove_directory(initialize()).await;
    }

    #[cfg(feature = "std")]
    #[test]
    async fn test_loader() {
        file_system::tests::test_loader(initialize());
    }
}
