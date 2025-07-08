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
    pub fn New() -> Self {
        Self(RwLock::new(Inner_type {
            devices: BTreeMap::new(),
            open_devices: BTreeMap::new(),
        }))
    }

    fn Borrow_mutable_inner_2_splitted<'b>(
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

    pub async fn Mount_device(
        &self,
        path: Path_owned_type,
        device: Device_type,
    ) -> Result_type<Inode_type> {
        let mut inner = self.0.write().await;

        let (Devices, _) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let Inode = get_new_inode(Devices)?;

        Devices.insert(Inode, (Internal_path_type::Owned(path), device));

        Ok(Inode)
    }

    pub async fn Mount_static_device(
        &self,
        path: &'a impl AsRef<Path_type>,
        device: Device_type,
    ) -> Result_type<Inode_type> {
        let mut inner = self.0.write().await;

        let (Devices, _) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let Inode = get_new_inode(Devices)?;

        Devices.insert(Inode, (Internal_path_type::Borrowed(path.as_ref()), device));

        Ok(Inode)
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
            .filter(|(_, (Device_path, _))| match Device_path {
                Internal_path_type::Borrowed(device_path) => {
                    device_path.Strip_prefix(path).is_some()
                }
                Internal_path_type::Owned(Device_path) => Device_path.Strip_prefix(path).is_some(),
            })
            .map(|(Inode, _)| *Inode)
            .collect())
    }

    pub async fn Open(
        &self,
        inode: Inode_type,
        task: Task_identifier_type,
        flags: Flags_type,
        underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (Devices, Open_pipes) = Self::Borrow_mutable_inner_2_splitted(&mut inner);

        let Device = Devices.get(&inode).ok_or(Error_type::Invalid_identifier)?;

        let Local_file_identifier = get_new_file_identifier(task, None, None, Open_pipes)?;

        Open_pipes.insert(
            Local_file_identifier,
            (Device.1.clone(), flags, underlying_file),
        );

        Ok(Local_file_identifier)
    }

    pub async fn Close(
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

    pub async fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()> {
        let mut inner = self.0.write().await;

        // Get all the keys of the open pipes that belong to the task
        let Keys = inner
            .open_devices
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the pipes corresponding to the keys
        for Key in Keys {
            if let Some((device, _, _)) = inner.open_devices.remove(&Key) {
                drop(device);
            }
        }

        Ok(())
    }

    pub async fn Duplicate(
        &self,
        file: Local_file_identifier_type,
        underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (Device, Flags, _) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let New_file = get_new_file_identifier(file.Split().0, None, None, &inner.open_devices)?;

        inner
            .open_devices
            .insert(New_file, (Device.clone(), Flags, underlying_file));

        Ok(New_file)
    }

    pub async fn Transfert(
        &self,
        new_task: Task_identifier_type,
        file: Local_file_identifier_type,
        underlying_file: Unique_file_identifier_type,
        new_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut inner = self.0.write().await;

        let (Device, Flags, _) = inner
            .open_devices
            .remove(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        let New_file = if let Some(New_file) = new_file {
            let file = Local_file_identifier_type::New(new_task, New_file);

            if inner.open_devices.contains_key(&file) {
                return Err(Error_type::Invalid_identifier);
            }

            file
        } else {
            get_new_file_identifier(new_task, None, None, &inner.open_devices)?
        };

        if inner
            .open_devices
            .insert(New_file, (Device, Flags, underlying_file))
            .is_some()
        {
            return Err(Error_type::Internal_error); // Should never happen
        }

        Ok(New_file)
    }

    pub async fn Remove(&self, Inode: Inode_type) -> Result_type<Device_inner_type<'a>> {
        self.0
            .write()
            .await
            .devices
            .remove(&Inode)
            .ok_or(Error_type::Invalid_inode)
    }

    pub async fn Read(
        &self,
        file: Local_file_identifier_type,
        buffer: &mut [u8],
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let inner = self.0.read().await;

        let (Device, Flags, Underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.get_mode().get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.get_status().get_non_blocking() {
            return Ok((Device.Read(buffer)?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            let Size = Device.Read(buffer)?;

            if Size != 0 {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn Read_line(
        &self,
        file: Local_file_identifier_type,
        buffer: &mut String,
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let inner = self.0.read().await;

        let (Device, Flags, Underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.get_mode().get_read() {
            return Err(Error_type::Invalid_mode);
        }

        buffer.clear();

        let Byte: &mut [u8] = &mut [0; 1];

        loop {
            // Wait for the device to be ready
            let Size = Device.Read(Byte)?;

            if Size == 0 {
                yield_now().await;
                continue;
            }

            if Byte[0] == b'\n' || Byte[0] == b'\r' {
                return Ok((Size, *Underlying_file));
            }

            buffer.push(Byte[0] as char);
        }
    }

    pub async fn Write(
        &self,
        file: Local_file_identifier_type,
        buffer: &[u8],
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let inner = self.0.read().await;

        let (Device, Flags, Underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.get_mode().get_write() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.get_status().get_non_blocking() {
            return Ok((Device.Write(buffer)?, *Underlying_file));
        }

        loop {
            // Wait for the device to be ready
            if Device.Write(buffer).is_ok() {
                return Ok((buffer.len().into(), *Underlying_file));
            }
        }
    }

    pub async fn Set_position(
        &self,
        file: Local_file_identifier_type,
        position: &Position_type,
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let inner = self.0.read().await;

        let (Device, _, Underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok((Device.Set_position(position)?, *Underlying_file))
    }

    pub async fn Flush(
        &self,
        file: Local_file_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let inner = self.0.read().await;

        let (Device, _, Underlying_file) = inner
            .open_devices
            .get(&file)
            .ok_or(Error_type::Invalid_identifier)?;

        Device.Flush()?;

        Ok(*Underlying_file)
    }

    pub async fn get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()
            .await
            .open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .get_mode())
    }

    pub async fn get_raw_device(&self, Inode: Inode_type) -> Result_type<Device_type> {
        Ok(self
            .0
            .read()
            .await
            .devices
            .get(&Inode)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .clone())
    }

    pub async fn is_a_terminal(&self, File: Local_file_identifier_type) -> Result_type<bool> {
        Ok(self
            .0
            .read()
            .await
            .open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .is_a_terminal())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use file_system::{Create_device, Position_type};

    use file_system::Memory_device_type;
    use task::Test;

    use super::*;

    pub const MEMORY_DEVICE_SIZE: usize = 1024;
    pub const MEMORY_DEVICE_BLOCK_SIZE: usize = 512;

    #[Test]
    async fn test_mount_device() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device)
            .await
            .unwrap();
        assert!(fs.get_raw_device(Inode).await.is_ok());
    }

    #[Test]
    async fn test_open_close_device() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();
        assert!(fs.Close(file_id).await.is_ok());
    }

    #[Test]
    async fn test_read_write_device() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::new(0),
                Mode_type::READ_WRITE.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        let write_data = b"Hello, world!";
        fs.Write(file_id, write_data).await.unwrap();

        fs.Set_position(file_id, &Position_type::Start(0))
            .await
            .unwrap();

        let mut read_data = vec![0; write_data.len()];
        fs.Read(file_id, &mut read_data).await.unwrap();

        assert_eq!(&read_data, write_data);
    }

    #[Test]
    async fn test_duplicate_file_identifier() {
        let File_system = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = File_system
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();

        let Underlying_file = 0_usize.into();

        let File = File_system
            .Open(
                Inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                Underlying_file,
            )
            .await
            .unwrap();

        let New_underlying_file = 1_usize.into();

        let New_file = File_system
            .Duplicate(File, New_underlying_file)
            .await
            .unwrap();

        assert_eq!(
            File_system.get_underlying_file(New_file).await.unwrap(),
            New_underlying_file
        );

        assert!(File_system.Close(New_file).await.is_ok());
    }

    #[Test]
    async fn test_transfert_file_identifier() {
        let File_system = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = File_system
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();

        let Task = Task_identifier_type::new(0);

        let File_identifier = File_system
            .Open(Inode, Task, Mode_type::READ_ONLY.into(), 0_usize.into())
            .await
            .unwrap();

        let New_task = Task_identifier_type::new(1);

        let New_file_identifier = File_system
            .Transfert(New_task, File_identifier, 0_usize.into(), None)
            .await
            .unwrap();

        assert_eq!(New_file_identifier.Split().0, New_task);

        File_system.Close(New_file_identifier).await.unwrap();
    }

    #[Test]
    async fn test_delete_device() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        assert!(fs.Remove(Inode).await.is_ok());
    }

    #[Test]
    async fn test_set_position() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        fs.Set_position(file_id, &Position_type::Start(10))
            .await
            .unwrap();
    }

    #[Test]
    async fn test_flush() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        assert!(fs.Flush(file_id).await.is_ok());
    }

    #[Test]
    async fn test_get_mode() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<MEMORY_DEVICE_BLOCK_SIZE>::New(
            MEMORY_DEVICE_SIZE
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::new(0),
                Mode_type::READ_ONLY.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        assert!(fs.get_mode(file_id).await.is_ok());
    }
}
