use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use Task::Task_identifier_type;
use Users::{
    Group_identifier_type, Root_group_identifier, Root_user_identifier, User_identifier_type,
};

use crate::{
    Error_type, File_identifier_type, File_system_traits, Flags_type, Path_type, Permissions_type,
    Position_type, Result_type, Size_type, Type_type,
};

use super::Device_trait;

struct Internal_device_type {
    pub Device: Arc<Box<dyn Device_trait>>,
    pub User: User_identifier_type,
    pub Group: User_identifier_type,
    pub Permissions: Permissions_type,
}

struct Inner_type {
    Devices: HashMap<&'static Path_type, Internal_device_type>,
    Opened_devices: BTreeMap<u32, (Arc<Box<dyn Device_trait>>, Flags_type)>,
}

pub struct File_system_type(RwLock<Inner_type>);

impl File_system_type {
    pub fn New() -> Self {
        Self(RwLock::new(Inner_type {
            Devices: HashMap::new(),
            Opened_devices: BTreeMap::new(),
        }))
    }

    fn Get_new_file_identifier(
        &self,
        Task: Task_identifier_type,
    ) -> Result_type<File_identifier_type> {
        let Start = Self::Get_local_file_identifier(Task, File_identifier_type::from(0));
        let End = Self::Get_local_file_identifier(Task, File_identifier_type::from(0xFFFF));

        for File_identifier in Start..=End {
            if !self.0.read()?.Opened_devices.contains_key(&File_identifier) {
                return Ok(File_identifier_type::from(File_identifier as u16));
                // Remove the task identifier and keep the file identifier.
            }
        }

        Err(Error_type::Too_many_open_files)
    }
}

impl File_system_traits for File_system_type {
    fn Exists(&self, Path: &dyn AsRef<Path_type>) -> Result_type<bool> {
        Ok(self.0.read()?.Devices.contains_key(Path.as_ref()))
    }

    fn Create_file(&self, _: &dyn AsRef<Path_type>) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    fn Open(
        &self,
        Task: Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: crate::Flags_type,
    ) -> Result_type<File_identifier_type> {
        let Opened_device = self
            .0
            .read()
            .unwrap()
            .Devices
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Device
            .clone();

        let File_identifier = self.Get_new_file_identifier(Task)?;

        self.0.write()?.Opened_devices.insert(
            Self::Get_local_file_identifier(Task, File_identifier),
            (Opened_device, Flags),
        );

        Ok(File_identifier)
    }

    fn Close(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()> {
        self.0
            .write()?
            .Opened_devices
            .remove(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?;

        Ok(())
    }

    fn Close_all(&self, Task: Task_identifier_type) -> Result_type<()> {
        self.0
            .write()?
            .Opened_devices
            .retain(|Key, _| Self::Decompose_local_file_identifier(*Key).0 != Task);

        Ok(())
    }

    fn Transfert_file_identifier(
        &self,
        Old_task: Task_identifier_type,
        New_task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<File_identifier_type> {
        let File_identifier = Self::Get_local_file_identifier(Old_task, File);

        let (Device, Flags) = self
            .0
            .write()?
            .Opened_devices
            .remove(&File_identifier)
            .ok_or(Error_type::Not_found)?;

        let New_file_identifier = self.Get_new_file_identifier(New_task)?;

        self.0.write()?.Opened_devices.insert(
            Self::Get_local_file_identifier(New_task, New_file_identifier),
            (Device, Flags),
        );

        Ok(New_file_identifier)
    }

    fn Delete(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()> {
        self.0
            .write()?
            .Devices
            .remove(Path.as_ref())
            .ok_or(Error_type::Not_found)?;

        Ok(())
    }

    fn Read(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        self.0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .0
            .Read(Buffer)
            .map(|Size| Size.into())
    }

    fn Write(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<Size_type> {
        self.0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .0
            .Write(Buffer)
            .map(|Size| Size.into())
    }

    fn Move(&self, _: &dyn AsRef<Path_type>, _: &dyn AsRef<Path_type>) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    fn Set_position(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Position: &Position_type,
    ) -> Result_type<Size_type> {
        self.0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .0
            .Set_position(Position)
            .map(|Size| Size.into())
    }

    fn Flush(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()> {
        self.0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .0
            .Flush()
    }

    fn Get_type(&self, _: &dyn AsRef<Path_type>) -> Result_type<Type_type> {
        Ok(Type_type::Character_device)
    }

    fn Get_size(&self, Path: &dyn AsRef<Path_type>) -> Result_type<Size_type> {
        self.0
            .read()?
            .Devices
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Device
            .Get_size()
            .map(|Size| Size.into())
    }

    fn Create_directory(&self, _: &dyn AsRef<Path_type>) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    fn Add_device(
        &self,
        Path: &'static dyn AsRef<Path_type>,
        Device: Box<dyn Device_trait>,
    ) -> Result_type<()> {
        let Inner = &mut self.0.write()?;

        if Inner.Devices.contains_key(Path.as_ref()) {
            return Err(Error_type::Already_exists);
        }

        Inner.Devices.insert(
            Path.as_ref(),
            Internal_device_type {
                Device: Arc::new(Device),
                User: Root_user_identifier,
                Group: Root_group_identifier,
                Permissions: Permissions_type::New_standard_file(),
            },
        );

        Ok(())
    }

    fn Set_permissions(
        &self,
        Path: &dyn AsRef<Path_type>,
        Permissions: Permissions_type,
    ) -> Result_type<()> {
        self.0
            .write()?
            .Devices
            .get_mut(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Permissions = Permissions;

        Ok(())
    }

    fn Get_permissions(&self, Path: &dyn AsRef<Path_type>) -> Result_type<Permissions_type> {
        Ok(self
            .0
            .read()?
            .Devices
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Permissions)
    }

    fn Set_owner(
        &self,
        Path: &dyn AsRef<Path_type>,
        User: Option<User_identifier_type>,
        Group: Option<Users::Group_identifier_type>,
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        let Device = Inner
            .Devices
            .get_mut(Path.as_ref())
            .ok_or(Error_type::Not_found)?;

        if let Some(User) = User {
            Device.User = User;
        }

        if let Some(Group) = Group {
            Device.Group = Group;
        }

        Ok(())
    }

    fn Get_owner(
        &self,
        Path: &dyn AsRef<Path_type>,
    ) -> Result_type<(User_identifier_type, Group_identifier_type)> {
        self.0
            .read()?
            .Devices
            .get(Path.as_ref())
            .map(|Device| (Device.User, Device.Group))
            .ok_or(Error_type::Not_found)
    }
}
