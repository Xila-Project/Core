use std::{collections::BTreeMap, sync::RwLock, time::Duration};

use Task::Task_identifier_type;

use crate::{
    Error_type, Flags_type, Get_new_file_identifier, Get_new_inode, Inode_type,
    Local_file_identifier_type, Mode_type, Result_type, Size_type,
};

use super::Device_type;

struct Inner_type {
    pub Devices: BTreeMap<Inode_type, Device_type>,
    pub Open_devices: BTreeMap<Local_file_identifier_type, (Device_type, Flags_type)>,
}

pub struct File_system_type(RwLock<Inner_type>);

impl File_system_type {
    pub fn New() -> Self {
        Self(RwLock::new(Inner_type {
            Devices: BTreeMap::new(),
            Open_devices: BTreeMap::new(),
        }))
    }

    fn Borrow_mutable_inner_2_splited(
        Inner: &mut Inner_type,
    ) -> (
        &mut BTreeMap<Inode_type, Device_type>,
        &mut BTreeMap<Local_file_identifier_type, (Device_type, Flags_type)>,
    ) {
        (&mut Inner.Devices, &mut Inner.Open_devices)
    }

    pub fn Mount_device(&self, Device: Device_type) -> Result_type<Inode_type> {
        let mut Inner = self.0.write()?;

        let (Devices, _) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let Inode = Get_new_inode(Devices)?;

        Devices.insert(Inode, Device);

        Ok(Inode)
    }

    pub fn Open(
        &self,
        Inode: Inode_type,
        Task: Task_identifier_type,
        Flags: Flags_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write()?;

        let (Devices, Open_pipes) = Self::Borrow_mutable_inner_2_splited(&mut Inner);

        let Device = Devices.get(&Inode).ok_or(Error_type::Invalid_identifier)?;

        let Local_file_identifier = Get_new_file_identifier(Task, Open_pipes)?;

        Open_pipes.insert(Local_file_identifier, (Device.clone(), Flags));

        Ok(Local_file_identifier)
    }

    pub fn Close(&self, File: Local_file_identifier_type) -> Result_type<()> {
        self.0
            .write()?
            .Open_devices
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(())
    }

    pub fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        // Get all the keys of the open pipes that belong to the task
        let Keys = Inner
            .Open_devices
            .keys()
            .filter(|Key| Key.Split().0 == Task)
            .cloned()
            .collect::<Vec<_>>();

        // Close all the pipes corresponding to the keys
        for Key in Keys {
            if let Some((Device, _)) = Inner.Open_devices.remove(&Key) {
                drop(Device);
            }
        }

        Ok(())
    }

    pub fn Duplicate_file_identifier(
        &self,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write()?;

        let (Device, Flags) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .clone();

        let New_file = Get_new_file_identifier(File.Split().0, &Inner.Open_devices)?;

        Inner
            .Open_devices
            .insert(New_file, (Device.clone(), Flags.clone()));

        Ok(New_file)
    }

    pub fn Transfert_file_identifier(
        &self,
        New_task: Task_identifier_type,
        File: Local_file_identifier_type,
    ) -> Result_type<Local_file_identifier_type> {
        let mut Inner = self.0.write()?;

        let (Device, Flags) = Inner
            .Open_devices
            .remove(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        let New_file = Get_new_file_identifier(New_task, &Inner.Open_devices)?;

        Inner.Open_devices.insert(New_file, (Device, Flags));

        Ok(New_file)
    }

    pub fn Remove(&self, Inode: Inode_type) -> Result_type<()> {
        self.0
            .write()?
            .Devices
            .remove(&Inode)
            .ok_or(Error_type::Invalid_identifier)?;

        Ok(())
    }

    pub fn Read(
        &self,
        File: Local_file_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        let Inner = self.0.read()?;

        let (Device, Flags) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_read() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Device.Read(Buffer);
        }

        loop {
            // Wait for the pipe to be ready
            if let Ok(Size) = Device.Read(Buffer) {
                return Ok(Size);
            }

            Task::Manager_type::Sleep(Duration::from_millis(1));
        }
    }

    pub fn Write(&self, File: Local_file_identifier_type, Buffer: &[u8]) -> Result_type<Size_type> {
        let Inner = self.0.read()?;

        let (Device, Flags) = Inner
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?;

        if !Flags.Get_mode().Get_write() {
            return Err(Error_type::Invalid_mode);
        }

        if Flags.Get_status().Get_non_blocking() {
            return Device.Write(Buffer);
        }

        loop {
            // Wait for the device to be ready
            if Device.Write(Buffer).is_ok() {
                return Ok(Buffer.len().into());
            }

            Task::Manager_type::Sleep(Duration::from_millis(1));
        }
    }

    pub fn Get_size(&self, File: Local_file_identifier_type) -> Result_type<Size_type> {
        Ok(self
            .0
            .read()?
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .Get_size()?)
    }

    pub fn Set_position(
        &self,
        File: Local_file_identifier_type,
        Position: &crate::Position_type,
    ) -> Result_type<Size_type> {
        self.0
            .read()?
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .Set_position(Position)
    }

    pub fn Flush(&self, File: Local_file_identifier_type) -> Result_type<()> {
        self.0
            .read()?
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .Flush()
    }

    pub fn Get_mode(&self, File: Local_file_identifier_type) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()?
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .1
            .Get_mode())
    }

    pub fn Get_raw_device(&self, Inode: Inode_type) -> Result_type<Device_type> {
        Ok(self
            .0
            .read()?
            .Devices
            .get(&Inode)
            .ok_or(Error_type::Invalid_identifier)?
            .clone())
    }

    pub fn Is_a_terminal(&self, File: Local_file_identifier_type) -> Result_type<bool> {
        Ok(self
            .0
            .read()?
            .Open_devices
            .get(&File)
            .ok_or(Error_type::Invalid_identifier)?
            .0
            .Is_a_terminal())
    }
}

#[cfg(test)]
mod tests {
    use crate::Create_device;
    use crate::Device_trait;
    use crate::Position_type;
    use crate::Tests::Memory_device_type;

    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use std::sync::RwLock;

    pub const Memory_device_size: usize = 1024;
    pub const Memory_device_block_size: usize = 512;

    #[test]
    fn Test_mount_device() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        assert!(fs.Get_raw_device(Inode).is_ok());
    }

    #[test]
    fn Test_open_close_device() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
            )
            .unwrap();
        assert!(fs.Close(file_id).is_ok());
    }

    #[test]
    fn Test_read_write_device() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_write.into(),
            )
            .unwrap();

        let write_data = b"Hello, world!";
        fs.Write(file_id, write_data).unwrap();

        fs.Set_position(file_id, &Position_type::Start(0)).unwrap();

        let mut read_data = vec![0; write_data.len()];
        fs.Read(file_id, &mut read_data).unwrap();

        assert_eq!(&read_data, write_data);
    }

    #[test]
    fn Test_duplicate_file_identifier() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
            )
            .unwrap();
        let new_file_id = fs.Duplicate_file_identifier(file_id).unwrap();

        assert!(fs.Close(new_file_id).is_ok());
    }

    #[test]
    fn Test_transfert_file_identifier() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
            )
            .unwrap();
        let new_file_id = fs
            .Transfert_file_identifier(Task_identifier_type::New(0), file_id)
            .unwrap();

        assert!(fs.Close(new_file_id).is_ok());
    }

    #[test]
    fn Test_delete_device() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        assert!(fs.Remove(Inode).is_ok());
    }

    #[test]
    fn Test_get_size() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
            )
            .unwrap();

        assert_eq!(Memory_device_size, fs.Get_size(file_id).unwrap().into());
    }

    #[test]
    fn Test_set_position() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
            )
            .unwrap();

        fs.Set_position(file_id, &Position_type::Start(10)).unwrap();
        assert_eq!(Memory_device_size, fs.Get_size(file_id).unwrap().into());
    }

    #[test]
    fn Test_flush() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
            )
            .unwrap();

        assert!(fs.Flush(file_id).is_ok());
    }

    #[test]
    fn Test_get_mode() {
        let fs = File_system_type::New();

        let Inode = fs
            .Mount_device(Create_device!(
                Memory_device_type::<Memory_device_block_size>::New(Memory_device_size)
            ))
            .unwrap();
        let file_id = fs
            .Open(
                Inode,
                Task_identifier_type::New(0),
                Mode_type::Read_only.into(),
            )
            .unwrap();

        assert!(fs.Get_mode(file_id).is_ok());
    }
}
