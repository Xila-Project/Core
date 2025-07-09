use alloc::{collections::BTreeMap, string::String, vec::Vec};

use futures::yield_now;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use task::Task_identifier_type;

use file_system::{
    get_new_file_identifier, get_new_inode, Device_type, Error_type, File_identifier_type,
    Flags_type, Inode_type, Local_file_identifier_type, Mode_type, Path_owned_type, Path_type,
    Position_type, Result_type, Size_type, Unique_file_identifier_type,
};

type Open_device_inner_type = (Device_type, Flags_type, Unique_file_identifier_type);

#[derive(Clone)]
pub enum Internal_path_type<'a> {
    Borrowed(&'a Path_type),
    Owned(Path_owned_type),
}

struct Inner_type<'a> {
    pub devices: BTreeMap<Inode_type, Device_inner_type<'a>>,
    pub open_devices: BTreeMap<Local_file_identifier_type, Open_device_inner_type>,
}

type Device_inner_type<'a> = (Internal_path_type<'a>, Device_type);

pub struct File_system_type<'a>(RwLock<CriticalSectionRawMutex, Inner_type<'a>>);

impl<'a> File_system_type<'a> {
    pub fn new() -> Self {
        Self(RwLock::new(Inner_type {
            devices: BTreeMap::new(),
            open_devices: BTreeMap::new(),
        }))
    }

    fn borrow_mutable_inner_2_splitted<'b>(
        inner: &'b mut Inner_type<'a>,
    ) -> (
        &'b mut BTreeMap<Inode_type, Device_inner_type<'a>>,
        &'b mut BTreeMap<Local_file_identifier_type, Open_device_inner_type>,
    ) {
        (&mut inner.devices, &mut inner.open_devices)
    }

    pub async fn get_underlying_file(
        &self,
        file: Local_file_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        Ok(self
            .0
            .read()
            .await
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .2)
    }

    pub async fn mount_device(
        &self,
        path: Path_owned_type,
        device: Device_type,
    ) -> Result_type<Inode_type> {
        let mut inner = self.0.write().await;

        let (devices, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let inode = get_new_inode(devices)?;

        devices.insert(inode, (Internal_path_type::Owned(path), device));

        Ok(inode)
    }

    pub async fn mount_static_device(
        &self,
        path: &'a impl AsRef<Path_type>,
        device: Device_type,
    ) -> Result_type<Inode_type> {
        let mut inner = self.0.write().await;

        let (devices, _) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let inode = get_new_inode(devices)?;

        devices.insert(inode, (Internal_path_type::Borrowed(path.as_ref()), device));

        Ok(inode)
    }

    pub async fn get_path_from_inode(
        &self,
        inode: Inode_type,
    ) -> Result_type<Internal_path_type<'a>> {
        Ok(self
            .0
            .read()
            .await
            .devices
            .get(&inode)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .clone())
    }

    pub async fn get_devices_from_path(
        &self,
        path: &'static Path_type,
    ) -> Result_type<Vec<Inode_type>> {
        Ok(self
            .0
            .read()
            .await
            .devices
            .iter()
            .filter(|(_, (device_path, _))| match device_path {
                Internal_path_type::Borrowed(device_path) => {
                    device_path.strip_prefix(path).is_some()
                }
                Internal_path_type::Owned(device_path) => device_path.strip_prefix(path).is_some(),
            })
            .map(|(inode, _)| *inode)
            .collect())
    }

    pub async fn open(
        &self,
        inode: Inode_type,
        task: Task_identifier_type,
        flags: Flags_type,
        underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (devices, open_pipes) = Self::borrow_mutable_inner_2_splitted(&mut inner);

        let device = devices.get(&inode).ok_or(Error_type::Invalid_identifier)?;

        let local_file_identifier = get_new_file_identifier(task, None, None, open_pipes)?;

        open_pipes.insert(
            local_file_identifier,
            (device.1.clone(), flags, underlying_file),
        );

        Ok(local_file_identifier)
    }

    pub async fn close(
        &self,
        file: Local_file_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let (_, _, underlying_file) = self
            .0
            .write()
            .await
            .open_devices
            .remove(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(underlying_file)
    }

    pub async fn close_all(&self, task: Task_identifier_type) -> Result_type<()> {
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
        file: Local_file_identifier_type,
        underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (device, flags, _) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let new_file = get_new_file_identifier(file.split().0, None, None, &inner.open_devices)?;

        inner
            .open_devices
            .insert(new_file, (device.clone(), flags, underlying_file));

        Ok(new_file)
    }

    pub async fn transfert(
        &self,
        new_task: Task_identifier_type,
        file: Local_file_identifier_type,
        underlying_file: Unique_file_identifier_type,
        new_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (device, flags, _) = inner
            .open_devices
            .remove(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        let new_file = if let Some(new_file) = new_file {
            let file = Local_file_identifier_type::new(new_task, new_file);

            if inner.open_devices.contains_key(&file) {
                return Err(Error_type::Invalid_identifier);
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
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(new_file)
    }

    pub async fn remove(&self, inode: Inode_type) -> Result_type<Device_inner_type<'a>> {
        self.0
            .write()
            .await
            .devices
            .remove(&inode)
            .ok_or(Error_type::Invalid_inode)
    }

    pub async fn read(
        &self,
        file: Local_file_identifier_type,
        buffer: &mut [u8],
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let inner = self.0.read().await;

        let (device, flags, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !flags.get_mode().get_read() {
            return Err(Error_type::Invalid_mode);
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
        file: Local_file_identifier_type,
        buffer: &mut String,
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let inner = self.0.read().await;

        let (device, flags, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !flags.get_mode().get_read() {
            return Err(Error_type::Invalid_mode);
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
        file: Local_file_identifier_type,
        buffer: &[u8],
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let inner = self.0.read().await;

        let (device, flags, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !flags.get_mode().get_write() {
            return Err(Error_type::Invalid_mode);
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
        file: Local_file_identifier_type,
        position: &Position_type,
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let inner = self.0.read().await;

        let (device, _, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok((device.set_position(position)?, *underlying_file))
    }

    pub async fn flush(
        &self,
        file: Local_file_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let inner = self.0.read().await;

        let (device, _, underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        device.flush()?;

        Ok(*underlying_file)
    }

    pub async fn get_mode(&self, file: Local_file_identifier_type) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()
            .await
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .get_mode())
    }

    pub async fn get_raw_device(&self, inode: Inode_type) -> Result_type<Device_type> {
        Ok(self
            .0
            .read()
            .await
            .devices
            .get(&inode)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .clone())
    }

    pub async fn is_a_terminal(&self, file: Local_file_identifier_type) -> Result_type<bool> {
        Ok(self
            .0
            .read()
            .await
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .is_a_terminal())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use file_system::{create_device, Position_type};

    use file_system::Memory_device_type;
    use task::Test;

    use super::*;

    pub const MEMORY_DEVICE_SIZE: usize = 1024;
    pub const MEMORY_DEVICE_BLOCK_SIZE: usize = 512;

    #[Test]
    async fn test_mount_device() {
        let fs = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device)
            .await
            .unwrap();
        assert!(fs.get_raw_device(inode).await.is_ok());
    }

    #[Test]
    async fn test_open_close_device() {
        let fs = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();
        assert!(fs.close(file_id).await.is_ok());
    }

    #[Test]
    async fn test_read_write_device() {
        let fs = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                Task_identifier_type::new(0),
                Mode_type::READ_WRITE.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        let write_data = b"Hello, world!";
        fs.write(file_id, write_data).await.unwrap();

        fs.set_position(file_id, &Position_type::Start(0))
            .await
            .unwrap();

        let mut read_data = vec![0; write_data.len()];
        fs.read(file_id, &mut read_data).await.unwrap();

        assert_eq!(&read_data, write_data);
    }

    #[Test]
    async fn test_duplicate_file_identifier() {
        let file_system = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
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
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
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

    #[Test]
    async fn test_transfert_file_identifier() {
        let file_system = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = file_system
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();

        let task = Task_identifier_type::new(0);

        let file_identifier = file_system
            .open(inode, task, Mode_type::READ_ONLY.into(), 0_usize.into())
            .await
            .unwrap();

        let new_task = Task_identifier_type::new(1);

        let new_file_identifier = file_system
            .transfert(new_task, file_identifier, 0_usize.into(), None)
            .await
            .unwrap();

        assert_eq!(new_file_identifier.Split().0, new_task);

        file_system.close(new_file_identifier).await.unwrap();
    }

    #[Test]
    async fn test_delete_device() {
        let fs = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        assert!(fs.remove(inode).await.is_ok());
    }

    #[Test]
    async fn test_set_position() {
        let fs = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        fs.set_position(file_id, &Position_type::Start(10))
            .await
            .unwrap();
    }

    #[Test]
    async fn test_flush() {
        let fs = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        assert!(fs.flush(file_id).await.is_ok());
    }

    #[Test]
    async fn test_get_mode() {
        let fs = File_system_type::new();

        let memory_device = create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::new(
            MEMORY_DEVICE_SIZE
        ));

        let inode = fs
            .mount_static_device(&"Memory_device", memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .open(
                inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        assert!(fs.get_mode(file_id).await.is_ok());
    }
}
