use alloc::{collections::BTreeMap, string::String, vec::Vec};

use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use Task::Task_identifier_type;

use File_system::{
    Device_type, Error_type, File_identifier_type, Flags_type, Get_new_file_identifier,
    Get_new_inode, Inode_type, Local_file_identifier_type, Mode_type, Path_owned_type, Path_type,
    Position_type, Result_type, Size_type, Unique_file_identifier_type,
};

type Open_device_inner_type = (Device_type, Flags_type, Unique_file_identifier_type);

#[derive(Clone)]
pub enum Internal_path_type<'a> {
    Borrowed(&'a Path_type),
    Owned(Path_owned_type),
}

struct Inner_type<'a> {
    pub Devices: BTreeMap<Inode_type, Device_inner_type<'a>>,
    pub Open_devices: BTreeMap<Local_file_identifier_type, Open_device_inner_type>,
}

type Device_inner_type<'a> = (Internal_path_type<'a>, Device_type);

pub struct File_system_type<'a>(RwLock<CriticalSectionRawMutex, Inner_type<'a>>);

impl<'a> File_system_type<'a> {
    pub fn New() -> Self {
        Self(RwLock::new(Inner_type {
            Devices: BTreeMap::new(),
            Open_devices: BTreeMap::new(),
        }))
    }

    fn Borrow_mutable_inner_2_splitted<'b>(
        Inner: &'b mut Inner_type<'a>,
    ) -> (
        &'b mut BTreeMap<Inode_type, Device_inner_type<'a>>,
        &'b mut BTreeMap<Local_file_identifier_type, Open_device_inner_type>,
    ) {
        (&mut Inner.Devices, &mut Inner.Open_devices)
    }

    pub async fn Get_underlying_file(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        Ok(self
            .0
            .read()
            .await
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .2)
    }

    pub async fn Mount_device(
        &self,
        Path: Path_owned_type,
        Device: Device_type,
    ) -> Result_type<Inode_type> {
        let mut Inner = self.0.write().await;

        let (Devices, _) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let Inode = Get_new_inode(Devices)?;

        Devices.insert(Inode, (Internal_path_type::Owned(Path), Device));

        Ok(Inode)
    }

    pub async fn Mount_static_device(
        &self,
        Path: &'a impl AsRef<Path_type>,
        Device: Device_type,
    ) -> Result_type<Inode_type> {
        let mut Inner = self.0.write().await;

        let (Devices, _) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let Inode = Get_new_inode(Devices)?;

        Devices.insert(Inode, (Internal_path_type::Borrowed(Path.as_ref()), Device));

        Ok(Inode)
    }

    pub async fn Get_path_from_inode(
        &self,
        Inode: Inode_type,
    ) -> Result_type<Internal_path_type<'a>> {
        Ok(self
            .0
            .read()
            .await
            .Devices
            .get(&Inode)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .clone())
    }

    pub async fn Get_devices_from_path(
        &self,
        Path: &'static Path_type,
    ) -> Result_type<Vec<Inode_type>> {
        Ok(self
            .0
            .read()
            .await
            .Devices
            .iter()
            .filter(|(_, (Device_path, _))| match Device_path {
                Internal_path_type::Borrowed(Device_path) => {
                    Device_path.Strip_prefix(Path).is_some()
                }
                Internal_path_type::Owned(Device_path) => Device_path.Strip_prefix(Path).is_some(),
            })
            .map(|(Inode, _)| *Inode)
            .collect())
    }

    pub async fn Open(
        &self,
        Inode: Inode_type,
        Task: Task_identifier_type,
        Flags: Flags_type,
        Underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write().await;

        let (Devices, Open_pipes) = Self::Borrow_mutable_inner_2_splitted(&mut Inner);

        let Device = Devices.get(&Inode).ok_or(Error_type::Invalid_identifier)?;

        let Local_file_identifier = Get_new_file_identifier(Task, None, None, Open_pipes)?;

        Open_pipes.insert(
            Local_file_identifier,
            (Device.1.clone(), Flags, Underlying_file),
        );

        Ok(Local_file_identifier)
    }

    pub async fn Close(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let (_, _, Underlying_file) = self
            .0
            .write()
            .await
            .Open_devices
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(Underlying_file)
    }

    pub async fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()> {
        let mut Inner = self.0.write().await;

        // Get all the keys of the open pipes that belong to the task
        let Keys = Inner
            .Open_devices
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the pipes corresponding to the keys
        for Key in Keys {
            if let Some((Device, _, _)) = Inner.Open_devices.remove(&Key) {
                drop(Device);
            }
        }

        Ok(())
    }

    pub async fn Duplicate(
        &self,
        File: Local_file_identifier_type,
        Underlying_file: Unique_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write().await;

        let (Device, Flags, _) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let New_file = Get_new_file_identifier(File.Split().0, None, None, &Inner.Open_devices)?;

        Inner
            .Open_devices
            .insert(New_file, (Device.clone(), Flags, Underlying_file));

        Ok(New_file)
    }

    pub async fn Transfert(
        &self,
        New_task: Task_identifier_type,
        File: Local_file_identifier_type,
        Underlying_file: Unique_file_identifier_type,
        New_file: Option<File_identifier_type>,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write().await;

        let (Device, Flags, _) = Inner
            .Open_devices
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let New_file = if let Some(New_file) = New_file {
            let File = Local_file_identifier_type::New(New_task, New_file);

            if Inner.Open_devices.contains_key(&File) {
                return Err(Error_type::Invalid_identifier);
            }

            File
        } else {
            Get_new_file_identifier(New_task, None, None, &Inner.Open_devices)?
        };

        if Inner
            .Open_devices
            .insert(New_file, (Device, Flags, Underlying_file))
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
            .Devices
            .remove(&Inode)
            .ok_or(Error_type::Invalid_inode)
    }

    pub async fn Read(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let Inner = self.0.read().await;

        let (Device, Flags, Underlying_file) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Ok((Device.Read(Buffer)?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            let Size = Device.Read(Buffer)?;

            if Size != 0 {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn Read_line(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut String,
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let Inner = self.0.read().await;

        let (Device, Flags, Underlying_file) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Ok((Device.Read_line(Buffer)?, *Underlying_file));
        }

        loop {
            // Wait for the pipe to be ready
            let Size = Device.Read_line(Buffer)?;

            if Size != 0 {
                return Ok((Size, *Underlying_file));
            }
        }
    }

    pub async fn Write(
        &self,
        File: Local_file_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let Inner = self.0.read().await;

        let (Device, Flags, Underlying_file) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_write() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Ok((Device.Write(Buffer)?, *Underlying_file));
        }

        loop {
            // Wait for the device to be ready
            if Device.Write(Buffer).is_ok() {
                return Ok((Buffer.len().into(), *Underlying_file));
            }
        }
    }

    pub async fn Set_position(
        &self,
        File: Local_file_identifier_type,
        Position: &Position_type,
    ) -> Result_type<(Size_type, Unique_file_identifier_type)> {
        let Inner = self.0.read().await;

        let (Device, _, Underlying_file) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok((Device.Set_position(Position)?, *Underlying_file))
    }

    pub async fn Flush(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Unique_file_identifier_type> {
        let Inner = self.0.read().await;

        let (Device, _, Underlying_file) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Device.Flush()?;

        Ok(*Underlying_file)
    }

    pub async fn Get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()
            .await
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .Get_mode())
    }

    pub async fn Get_raw_device(&self, Inode: Inode_type) -> Result_type<Device_type> {
        Ok(self
            .0
            .read()
            .await
            .Devices
            .get(&Inode)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .clone())
    }

    pub async fn Is_a_terminal(&self, File: Local_file_identifier_type) -> Result_type<bool> {
        Ok(self
            .0
            .read()
            .await
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .Is_a_terminal())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use File_system::{Create_device, Position_type};

    use File_system::Memory_device_type;
    use Task::Test;

    use super::*;

    pub const Memory_device_size: usize = 1024;
    pub const Memory_device_block_size: usize = 512;

    #[Test]
    async fn Test_mount_device() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device)
            .await
            .unwrap();
        assert!(fs.Get_raw_device(Inode).await.is_ok());
    }

    #[Test]
    async fn Test_open_close_device() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
                0_usize.into(),
            )
            .await
            .unwrap();
        assert!(fs.Close(file_id).await.is_ok());
    }

    #[Test]
    async fn Test_read_write_device() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_write.into(),
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
    async fn Test_duplicate_file_identifier() {
        let File_system = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = File_system
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();

        let Underlying_file = 0_usize.into();

        let File = File_system
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
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
            File_system.Get_underlying_file(New_file).await.unwrap(),
            New_underlying_file
        );

        assert!(File_system.Close(New_file).await.is_ok());
    }

    #[Test]
    async fn Test_transfert_file_identifier() {
        let File_system = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = File_system
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();

        let Task = Task_identifier_type::New(0);

        let File_identifier = File_system
            .Open(Inode, Task, Mode_type::Read_only.into(), 0_usize.into())
            .await
            .unwrap();

        let New_task = Task_identifier_type::New(1);

        let New_file_identifier = File_system
            .Transfert(New_task, File_identifier, 0_usize.into(), None)
            .await
            .unwrap();

        assert_eq!(New_file_identifier.Split().0, New_task);

        File_system.Close(New_file_identifier).await.unwrap();
    }

    #[Test]
    async fn Test_delete_device() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        assert!(fs.Remove(Inode).await.is_ok());
    }

    #[Test]
    async fn Test_set_position() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        fs.Set_position(file_id, &Position_type::Start(10))
            .await
            .unwrap();
    }

    #[Test]
    async fn Test_flush() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        assert!(fs.Flush(file_id).await.is_ok());
    }

    #[Test]
    async fn Test_get_mode() {
        let fs = File_system_type::New();

        let Memory_device = Create_device!(Memory_device_type::<Memory_device_block_size>::New(
            Memory_device_size
        ));

        let Inode = fs
            .Mount_static_device(&"Memory_device", Memory_device.clone())
            .await
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
                0_usize.into(),
            )
            .await
            .unwrap();

        assert!(fs.Get_mode(file_id).await.is_ok());
    }
}
