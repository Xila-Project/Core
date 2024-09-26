use std::{
    collections::{BTreeMap, HashMap},
    sync::{atomic::{AtomicUsize, Ordering}, RwLock},
};

use Task::Task_identifier_type;
use Users::{Root_group_identifier, Root_user_identifier, User_identifier_type};

use crate::{
    Error_type, File_identifier_inner_type, File_identifier_type, File_system_identifier_type,
    File_system_traits, Flags_type, Mode_type, Path_type, Permissions_type, Position_type,
    Result_type, Size_type, Statistics_type, Type_type,
};

use super::{Device_trait, Device_type};

enum Inner_item_type {
    Device(Device_type),
    Directory(AtomicUsize),
}

impl Clone for Inner_item_type {
    fn clone(&self) -> Self {
        match self {
            Self::Device(Device) => Self::Device(Device.clone()),
            Self::Directory(Count) => Self::Directory(AtomicUsize::new(Count.load(Ordering::Acquire))),
        }
    }
    
}

struct Inner_type {
    Devices: HashMap<&'static Path_type, Device_type>,
    Opened_devices: BTreeMap<usize, (Inner_item_type, Flags_type)>,
}

pub struct File_system_type(RwLock<Inner_type>);

impl File_system_type {
    pub fn New() -> Self {
        Self(RwLock::new(Inner_type {
            Devices: HashMap::new(),
            Opened_devices: BTreeMap::new(),
        }))
    }

    fn Get_new_file_identifier<T>(
        &self,
        Task: Task_identifier_type,
        Opened_devices: &BTreeMap<usize, T>,
    ) -> Result_type<File_identifier_type> {
        let Start = Self::Get_local_file_identifier(Task, File_identifier_type::from(0));
        let End = Self::Get_local_file_identifier(Task, File_identifier_type::from(0xFFFF));

        for File_identifier in Start..=End {
            if !Opened_devices.contains_key(&File_identifier) {
                return Ok(File_identifier_type::from(
                    File_identifier as File_identifier_inner_type,
                ));
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

    fn Open(
        &self,
        Task: Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: crate::Flags_type,
    ) -> Result_type<File_identifier_type> {
        let mut Inner = self.0.write()?;

        let Opened_device = Inner
            .Devices
            .get(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .clone();

        let File_identifier = self.Get_new_file_identifier(Task, &Inner.Opened_devices)?;

        let Local_file_identifier = Self::Get_local_file_identifier(Task, File_identifier);

        // - If the file is a directory
        if Flags.Get_open().Get_directory() {
            for (Device, _) in Inner.Opened_devices.values_mut() {
                if let Inner_item_type::Directory(Count) = Device {
                    Count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }

            Inner.Opened_devices.insert(
                Local_file_identifier,
                (Inner_item_type::Directory(AtomicUsize::new(0)), Flags),
            );

            Ok(File_identifier)
        }
        // - If the file is a device
        else {
            Inner.Opened_devices.insert(
                Local_file_identifier,
                (Inner_item_type::Device(Opened_device), Flags),
            );

            Ok(File_identifier)
        }
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
        New_file_identifier: Option<File_identifier_type>,
    ) -> Result_type<File_identifier_type> {
        let File_identifier = Self::Get_local_file_identifier(Old_task, File);

        let mut Inner = self.0.write()?;

        let New_file_identifier = if let Some(New_file_identifier) = New_file_identifier {
            New_file_identifier
        } else {
            self.Get_new_file_identifier(New_task, &Inner.Opened_devices)?
        };

        let (Device, Flags) = Inner
            .Opened_devices
            .remove(&File_identifier)
            .ok_or(Error_type::Not_found)?;

        let Local_file_identifier = Self::Get_local_file_identifier(New_task, New_file_identifier);

        Inner
            .Opened_devices
            .insert(Local_file_identifier, (Device, Flags));

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
        match &self
            .0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .0
        {
            Inner_item_type::Device(Device) => Device.Read(Buffer).map(|Size| Size.into()),
            Inner_item_type::Directory(_) => Err(Error_type::Invalid_file),
        }
    }

    fn Write(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<Size_type> {
        match &self
            .0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .0
        {
            Inner_item_type::Device(Device) => Device.Write(Buffer).map(|Size| Size.into()),
            Inner_item_type::Directory(_) => Err(Error_type::Invalid_file),
        }
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
        match &self
            .0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .0
        {
            Inner_item_type::Device(Device) => {
                Device.Set_position(Position).map(|Size| Size.into())
            }
            Inner_item_type::Directory(_) => Err(Error_type::Unsupported_operation),
        }
    }

    fn Flush(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()> {
        match &self
            .0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .0
        {
            Inner_item_type::Device(Device) => Device.Flush(),
            Inner_item_type::Directory(_) => Err(Error_type::Unsupported_operation),
        }
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
            Device_type::New(
                Device,
                Root_user_identifier,
                Root_group_identifier,
                Permissions_type::New_default(Type_type::Character_device),
            ),
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
            .Set_permissions(Permissions)
    }

    fn Set_owner(
        &self,
        Path: &dyn AsRef<Path_type>,
        User: Option<User_identifier_type>,
        Group: Option<Users::Group_identifier_type>,
    ) -> Result_type<()> {
        self.0
            .write()?
            .Devices
            .get_mut(Path.as_ref())
            .ok_or(Error_type::Not_found)?
            .Set_owner(User, Group)
    }

    fn Get_statistics(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        File_system: File_system_identifier_type,
    ) -> Result_type<Statistics_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        match &self
            .0
            .read()?
            .Opened_devices
            .get(&Local_file_identifier)
            .ok_or(Error_type::Not_found)?
            .0
        {
            Inner_item_type::Device(Device) => {
                Device.Get_statistics(File_system, Local_file_identifier as u64)
            }
            Inner_item_type::Directory(_) => Err(Error_type::Unsupported_operation),
        }
    }

    fn Get_mode(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<Mode_type> {
        Ok(self
            .0
            .read()?
            .Opened_devices
            .get(&Self::Get_local_file_identifier(Task, File))
            .ok_or(Error_type::Not_found)?
            .1
            .Get_mode())
    }

    fn Duplicate_file_identifier(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<File_identifier_type> {
        let mut Inner = self.0.write()?;

        let File_identifier = Self::Get_local_file_identifier(Task, File);

        let (Device, Flags) = Inner
            .Opened_devices
            .get(&File_identifier)
            .ok_or(Error_type::Not_found)?
            .clone();

        let New_file_identifier = self.Get_new_file_identifier(Task, &Inner.Opened_devices)?;

        let Local_file_identifier = Self::Get_local_file_identifier(Task, New_file_identifier);

        Inner
            .Opened_devices
            .insert(Local_file_identifier, (Device, Flags));

        Ok(New_file_identifier)
    }
}
