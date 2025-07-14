use alloc::{collections::BTreeMap, string::String, vec::Vec};

use futures::yield_now;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use task::TaskIdentifier;

use file_system::{
    get_new_file_identifier, get_new_inode, DeviceType, Error, FileIdentifier, Flags, Inode,
    LocalFileIdentifier, Mode, Path, PathOwned, Position, Result, Size, UniqueFileIdentifier,
};

type OpenDeviceInnerType = (DeviceType, Flags, UniqueFileIdentifier);

#[derive(Clone)]
pub enum InternalPathType<'a> {
    Borrowed(&'a Path),
    Owned(PathOwned),
}

struct InnerType<'a> {
    pub devices: BTreeMap<Inode, DeviceInnerType<'a>>,
    pub open_devices: BTreeMap<LocalFileIdentifier, OpenDeviceInnerType>,
}

type DeviceInnerType<'a> = (InternalPathType<'a>, DeviceType);

pub struct FileSystem<'a>(RwLock<CriticalSectionRawMutex, InnerType<'a>>);

impl<'a> FileSystem<'a> {
    pub fn new() -> Self {
        Self(RwLock::new(InnerType {
            devices: BTreeMap::new(),
            open_devices: BTreeMap::new(),
        }))
    }

    fn borrow_mutable_inner_2_splitted<'b>(
        inner: &'b mut InnerType<'a>,
    ) -> (
        &'b mut BTreeMap<Inode, DeviceInnerType<'a>>,
        &'b mut BTreeMap<LocalFileIdentifier, OpenDeviceInnerType>,
    ) {
        (&mut inner.devices, &mut inner.open_devices)
    }

    pub async fn get_underlying_file(
        &self,
        file: LocalFileIdentifier,
    ) -> Result<UniqueFileIdentifier> {
        Ok(self
            .0
            .read()
            .await
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .2)
    }

    pub async fn mount_device(&self, path: PathOwned, device: DeviceType) -> Result<Inode> {
        let mut inner = self.0.write().await;

        let (devices, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let inode = get_new_inode(devices)?;

        devices.insert(inode, (InternalPathType::Owned(path), device));

        Ok(inode)
    }

    pub async fn mount_static_device(
        &self,
        path: &'a impl AsRef<Path>,
        device: DeviceType,
    ) -> Result<Inode> {
        let mut inner = self.0.write().await;

        let (devices, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let inode = get_new_inode(devices)?;

        devices.insert(inode, (InternalPathType::Borrowed(path.as_ref()), device));

        Ok(inode)
    }

    pub async fn get_path_from_inode(&self, inode: Inode) -> Result<InternalPathType<'a>> {
        Ok(self
            .0
            .read()
            .await
            .devices
            .get(&inode)
            .ok_or(Error::InvalidIdentifier)?
            .0
            .clone())
    }

    pub async fn get_devices_from_path(&self, path: &'static Path) -> Result<Vec<Inode>> {
        Ok(self
            .0
            .read()
            .await
            .devices
            .iter()
            .filter(|(_, (device_path, _))| match device_path {
                InternalPathType::Borrowed(device_path) => device_path.strip_prefix(path).is_some(),
                InternalPathType::Owned(device_path) => device_path.strip_prefix(path).is_some(),
            })
            .map(|(inode, _)| *inode)
            .collect())
    }

    pub async fn open(
        &self,
        inode: Inode,
        task: TaskIdentifier,
        flags: Flags,
        underlying_file: UniqueFileIdentifier,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.0.write().await;

        let (devices, open_pipes) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let device = devices.get(&inode).ok_or(Error::InvalidIdentifier)?;

        let local_file_identifier = get_new_file_identifier(task, None, None, open_pipes)?;

        open_pipes.insert(
            local_file_identifier,
            (device.1.clone(), flags, underlying_file),
        );

        Ok(local_file_identifier)
    }

    pub async fn close(&self, file: LocalFileIdentifier) -> Result<UniqueFileIdentifier> {
        let (_, _, underlying_file) = self
            .0
            .write()
            .await
            .open_devices
            .remove(&file)
            .ok_or(Error::InvalidIdentifier)?;

        Ok(underlying_file)
    }

    pub async fn close_all(&self, task: TaskIdentifier) -> Result<()> {
        let mut inner = self.0.write().await;

        // Get all the keys of the open pipes that belong to the task
        let keys = inner
            .open_devices
            .keys()
            .filter(|key| key.split().0 == task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the pipes corresponding to the keys
        for key in keys {
            if let Some((device, _, _)) = inner.open_devices.remove(&key) {
                drop(device);
            }
        }

        Ok(())
    }

    pub async fn duplicate(
        &self,
        file: LocalFileIdentifier,
        underlying_file: UniqueFileIdentifier,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.0.write().await;

        let (device, flags, _) = inner
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .clone();

        let new_file = get_new_file_identifier(file.split().0, None, None, &inner.open_devices)?;

        inner
            .open_devices
            .insert(new_file, (device.clone(), flags, underlying_file));

        Ok(new_file)
    }

    pub async fn transfert(
        &self,
        new_task: TaskIdentifier,
        file: LocalFileIdentifier,
        underlying_file: UniqueFileIdentifier,
        new_file: Option<FileIdentifier>,
    ) -> Result<LocalFileIdentifier> {
        let mut inner = self.0.write().await;

        let (device, flags, _) = inner
            .open_devices
            .remove(&file)
            .ok_or(Error::InvalidIdentifier)?;

        let new_file = if let Some(new_file) = new_file {
            let file = LocalFileIdentifier::new(new_task, new_file);

            if inner.open_devices.contains_key(&file) {
                return Err(Error::InvalidIdentifier);
            }

            file
        } else {
            get_new_file_identifier(new_task, None, None, &inner.open_devices)?
        };

        if inner
            .open_devices
            .insert(new_file, (device, flags, underlying_file))
            .is_some()
        {
            return Err(Error::InternalError); // Should never happen
        }

        Ok(new_file)
    }

    pub async fn remove(&self, inode: Inode) -> Result<DeviceInnerType<'a>> {
        self.0
            .write()
            .await
            .devices
            .remove(&inode)
            .ok_or(Error::InvalidInode)
    }

    pub async fn read(
        &self,
        file: LocalFileIdentifier,
        buffer: &mut [u8],
    ) -> Result<(Size, UniqueFileIdentifier)> {
        let inner = self.0.read().await;

        let (device, flags, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?;

        if !flags.get_mode().get_read() {
            return Err(Error::InvalidMode);
        }

        if flags.get_status().get_non_blocking() {
            return Ok((device.read(buffer)?, *underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            let size = device.read(buffer)?;

            if size != 0 {
                return Ok((size, *underlying_file));
            }
        }
    }

    pub async fn read_line(
        &self,
        file: LocalFileIdentifier,
        buffer: &mut String,
    ) -> Result<(Size, UniqueFileIdentifier)> {
        let inner = self.0.read().await;

        let (device, flags, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?;

        if !flags.get_mode().get_read() {
            return Err(Error::InvalidMode);
        }

        buffer.clear();

        let byte: &mut [u8] = &mut [0; 1];

        loop {
            // Wait for the device to be ready
            let size = device.read(byte)?;

            if size == 0 {
                yield_now().await;
                continue;
            }

            if byte[0] == b'\n' || byte[0] == b'\r' {
                return Ok((size, *underlying_file));
            }

            buffer.push(byte[0] as char);
        }
    }

    pub async fn write(
        &self,
        file: LocalFileIdentifier,
        buffer: &[u8],
    ) -> Result<(Size, UniqueFileIdentifier)> {
        let inner = self.0.read().await;

        let (device, flags, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?;

        if !flags.get_mode().get_write() {
            return Err(Error::InvalidMode);
        }

        if flags.get_status().get_non_blocking() {
            return Ok((device.write(buffer)?, *underlying_file));
        }

        loop {
            // Wait for the device to be ready
            if device.write(buffer).is_ok() {
                return Ok((buffer.len().into(), *underlying_file));
            }
        }
    }

    pub async fn set_position(
        &self,
        file: LocalFileIdentifier,
        position: &Position,
    ) -> Result<(Size, UniqueFileIdentifier)> {
        let inner = self.0.read().await;

        let (device, _, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?;

        Ok((device.set_position(position)?, *underlying_file))
    }

    pub async fn flush(&self, file: LocalFileIdentifier) -> Result<UniqueFileIdentifier> {
        let inner = self.0.read().await;

        let (device, _, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?;

        device.flush()?;

        Ok(*underlying_file)
    }

    pub async fn get_mode(&self, file: LocalFileIdentifier) -> Result<Mode> {
        Ok(self
            .0
            .read()
            .await
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .1
            .get_mode())
    }

    pub async fn get_raw_device(&self, inode: Inode) -> Result<DeviceType> {
        Ok(self
            .0
            .read()
            .await
            .devices
            .get(&inode)
            .ok_or(Error::InvalidIdentifier)?
            .1
            .clone())
    }

    pub async fn is_a_terminal(&self, file: LocalFileIdentifier) -> Result<bool> {
        Ok(self
            .0
            .read()
            .await
            .open_devices
            .get(&file)
            .ok_or(Error::InvalidIdentifier)?
            .0
            .is_a_terminal())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use alloc::vec;
    use file_system::{create_device, Position};

    use file_system::MemoryDevice;
    use task::test;

    use super::*;

    pub const MEMORY_DEVICE_SIZE: usize = 1024;
    pub const MEMORY_DEVICE_BLOCK_SIZE: usize = 512;

    #[test]
    async fn test_mount_device() {
        let fs = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device)
            .await
            .unwrap();
        assert!(fs.get_raw_device(inode).await.is_ok());
    }

    #[test]
    async fn test_open_close_device() {
        let fs = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                TaskIdentifier::new(0),
                Mode::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();
        assert!(fs.close(file_id).await.is_ok());
    }

    #[test]
    async fn test_read_write_device() {
        let fs = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                TaskIdentifier::new(0),
                Mode::READ_WRITE.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        let write_data = b"Hello, world!";
        fs.write(file_id, write_data).await.unwrap();

        fs.set_position(file_id, &Position::Start(0)).await.unwrap();

        let mut read_data = vec![0; write_data.len()];
        fs.read(file_id, &mut read_data).await.unwrap();

        assert_eq!(&read_data, write_data);
    }

    #[test]
    async fn test_duplicate_file_identifier() {
        let file_system = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = file_system
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();

        let underlying_file = 0_usize.into();

        let file = file_system
            .open(
                inode,
                TaskIdentifier::new(0),
                Mode::READ_ONLY.into(),
                underlying_file,
            )
            .await
            .unwrap();

        let new_underlying_file = 1_usize.into();

        let new_file = file_system
            .duplicate(file, new_underlying_file)
            .await
            .unwrap();

        assert_eq!(
            file_system.get_underlying_file(new_file).await.unwrap(),
            new_underlying_file
        );

        assert!(file_system.close(new_file).await.is_ok());
    }

    #[test]
    async fn test_transfert_file_identifier() {
        let file_system = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = file_system
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();

        let task = TaskIdentifier::new(0);

        let file_identifier = file_system
            .open(inode, task, Mode::READ_ONLY.into(), 0_usize.into())
            .await
            .unwrap();

        let new_task = TaskIdentifier::new(1);

        let new_file_identifier = file_system
            .transfert(new_task, file_identifier, 0_usize.into(), None)
            .await
            .unwrap();

        assert_eq!(new_file_identifier.split().0, new_task);

        file_system.close(new_file_identifier).await.unwrap();
    }

    #[test]
    async fn test_delete_device() {
        let fs = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        assert!(fs.remove(inode).await.is_ok());
    }

    #[test]
    async fn test_set_position() {
        let fs = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                TaskIdentifier::new(0),
                Mode::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        fs.set_position(file_id, &Position::Start(10))
            .await
            .unwrap();
    }

    #[test]
    async fn test_flush() {
        let fs = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                TaskIdentifier::new(0),
                Mode::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        assert!(fs.flush(file_id).await.is_ok());
    }

    #[test]
    async fn test_get_mode() {
        let fs = FileSystem::new();

        let memory_device = create_device!(MemoryDevice::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                TaskIdentifier::new(0),
                Mode::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        assert!(fs.get_mode(file_id).await.is_ok());
    }
}
