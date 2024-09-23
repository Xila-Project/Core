use std::sync::{Arc, RwLock};

use Users::{Group_identifier_type, User_identifier_type};

use crate::{
    File_system_identifier_type, Get_now, Permissions_type, Position_type, Result_type,
    Statistics_type, Time_type, Type_type,
};

pub trait Device_trait: Send + Sync {
    fn Read(&self, Buffer: &mut [u8]) -> Result_type<usize>;

    fn Write(&self, Buffer: &[u8]) -> Result_type<usize>;

    fn Get_size(&self) -> Result_type<usize>;

    fn Set_position(&self, Position: &Position_type) -> Result_type<usize>;

    fn Flush(&self) -> Result_type<()>;
}

struct Inner_type {
    Device: Box<dyn Device_trait>,
    User: User_identifier_type,
    Group: Group_identifier_type,
    Permissions: Permissions_type,
    Access_time: Time_type,
    Modification_time: Time_type,
    Status_change_time: Time_type,
}

#[derive(Clone)]
pub struct Device_type(Arc<RwLock<Inner_type>>);

impl Device_type {
    pub fn New(
        Device: Box<dyn Device_trait>,
        User: User_identifier_type,
        Group: Group_identifier_type,
        Permissions: Permissions_type,
    ) -> Self {
        Device_type(Arc::new(RwLock::new(Inner_type {
            Device,
            User,
            Group,
            Permissions,
            Access_time: Get_now(),
            Modification_time: Get_now(),
            Status_change_time: Get_now(),
        })))
    }

    pub fn Read(&self, Buffer: &mut [u8]) -> Result_type<usize> {
        let mut Inner = self.0.write()?;
        Inner.Access_time = Get_now();
        Inner.Device.Read(Buffer)
    }

    pub fn Write(&self, Buffer: &[u8]) -> Result_type<usize> {
        let mut Inner = self.0.write()?;
        Inner.Access_time = Get_now();
        Inner.Modification_time = Get_now();
        Inner.Device.Write(Buffer)
    }

    pub fn Set_position(&self, Position: &Position_type) -> Result_type<usize> {
        let mut Inner = self.0.write()?;
        Inner.Access_time = Get_now();
        Inner.Device.Set_position(Position)
    }

    pub fn Flush(&self) -> Result_type<()> {
        let mut Inner = self.0.write()?;
        Inner.Access_time = Get_now();
        Inner.Device.Flush()
    }

    pub fn Set_permissions(&self, Permissions: Permissions_type) -> Result_type<()> {
        let mut Inner = self.0.write()?;
        Inner.Permissions = Permissions;
        Inner.Status_change_time = Get_now();
        Ok(())
    }

    pub fn Set_owner(
        &self,
        User: Option<User_identifier_type>,
        Group: Option<Group_identifier_type>,
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;
        if let Some(User) = User {
            Inner.User = User;
        }
        if let Some(Group) = Group {
            Inner.Group = Group;
        }

        Inner.Status_change_time = Get_now();
        Ok(())
    }

    pub fn Get_statistics(
        &self,
        File_system: File_system_identifier_type,
        File: u64,
    ) -> Result_type<Statistics_type> {
        let Inner = self.0.read()?;
        Ok(Statistics_type::New(
            File_system,
            File,
            1,
            Inner.Device.Get_size()?.into(),
            Inner.Access_time,
            Inner.Modification_time,
            Inner.Status_change_time,
            Type_type::Character_device,
        ))
    }
}
