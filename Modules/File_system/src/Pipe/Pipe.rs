use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use Users::{Group_identifier_type, User_identifier_type};

use crate::{
    Error_type, File_system_identifier_type, Get_now, Inode_type, Permissions_type, Result_type,
    Statistics_type, Time_type, Type_type,
};

#[derive(Debug)]
struct Inner_type {
    pub Buffer: VecDeque<u8>,
    pub User: User_identifier_type,
    pub Group: Group_identifier_type,
    pub Permissions: Permissions_type,
    pub Access_time: Time_type,
    pub Modification_time: Time_type,
    pub Status_change_time: Time_type,
}

/// A pipe is a FIFO (ring) buffer that can be used to communicate between tasks.
#[derive(Debug, Clone)]
pub struct Pipe_type(Arc<RwLock<Inner_type>>);

impl Pipe_type {
    /// Create a new pipe with a buffer of the specified size.
    pub fn New(
        Buffer_size: usize,
        User: User_identifier_type,
        Group: Group_identifier_type,
        Permissions: Permissions_type,
    ) -> Self {
        Pipe_type(Arc::new(RwLock::new(Inner_type {
            Buffer: VecDeque::with_capacity(Buffer_size),
            User,
            Group,
            Permissions,
            Access_time: Get_now(),
            Modification_time: Get_now(),
            Status_change_time: Get_now(),
        })))
    }

    pub fn Write(&self, Data: &[u8]) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        let Buffer = &mut Inner.Buffer;

        if Data.len() > Buffer.capacity() - Buffer.len() {
            return Err(Error_type::Ressource_busy);
        }

        for Byte in Data {
            Buffer.push_back(*Byte);
        }

        Inner.Modification_time = Get_now();

        Ok(())
    }

    pub fn Read(&self, Data: &mut [u8]) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        let Buffer = &mut Inner.Buffer;

        if Data.len() > Buffer.len() {
            return Err(Error_type::Ressource_busy);
        }

        for Byte in Data {
            *Byte = Buffer.pop_front().unwrap();
        }

        Inner.Access_time = Get_now();

        Ok(())
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
            Inode_type::New(File),
            1,
            Inner.Buffer.len().into(),
            Inner.Access_time,
            Inner.Modification_time,
            Inner.Status_change_time,
            Type_type::Pipe,
        ))
    }
}
