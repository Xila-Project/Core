use Users::{Group_identifier_type, User_identifier_type};

use crate::{File_system_identifier_type, Size_type, Time_type, Type_type};

use super::{Inode_type, Permissions_type};

/// Statistics of a file.
///
/// This type contains information about a file, such as its size, inode, etc.
///
/// # Fields
///
/// * `File_system`: The file system the file is on.
/// * `Inode`: The inode of the file.
/// * `Links`: The number of hard links to the file.
/// * `Size`: The size of the file.
/// * `Last_access`: The last time the file was accessed.
/// * `Last_modification`: The last time the file was modified.
/// * `Last_status_change`: The last time the file's status was changed.
/// * `Type`: The type of the file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Statistics_type {
    File_system: File_system_identifier_type,
    Inode: Inode_type,
    Links: u64,
    Size: Size_type,
    Last_access: Time_type,
    Last_modification: Time_type,
    Last_status_change: Time_type,
    Type: Type_type,
    Permissions: Permissions_type,
    User: User_identifier_type,
    Group: Group_identifier_type,
}

impl Statistics_type {
    #[allow(clippy::too_many_arguments)]
    pub fn New(
        File_system: File_system_identifier_type,
        Inode: Inode_type,
        Links: u64,
        Size: Size_type,
        Last_access: Time_type,
        Last_modification: Time_type,
        Last_status_change: Time_type,
        Type: Type_type,
        Permissions: Permissions_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
    ) -> Self {
        Statistics_type {
            File_system,
            Inode,
            Links,
            Size,
            Last_access,
            Last_modification,
            Last_status_change,
            Type,
            Permissions,
            User,
            Group,
        }
    }

    pub const fn Get_file_system(&self) -> File_system_identifier_type {
        self.File_system
    }

    pub const fn Get_inode(&self) -> Inode_type {
        self.Inode
    }

    pub const fn Get_links(&self) -> u64 {
        self.Links
    }

    pub const fn Get_size(&self) -> Size_type {
        self.Size
    }

    pub const fn Get_last_access(&self) -> Time_type {
        self.Last_access
    }

    pub const fn Get_last_modification(&self) -> Time_type {
        self.Last_modification
    }

    pub const fn Get_last_status_change(&self) -> Time_type {
        self.Last_status_change
    }

    pub const fn Get_type(&self) -> Type_type {
        self.Type
    }

    pub const fn Get_permissions(&self) -> Permissions_type {
        self.Permissions
    }

    pub const fn Get_user(&self) -> User_identifier_type {
        self.User
    }

    pub const fn Get_group(&self) -> Group_identifier_type {
        self.Group
    }

    pub fn Set_file_system(&mut self, File_system: File_system_identifier_type) -> &mut Self {
        self.File_system = File_system;
        self
    }

    pub fn Set_inode(&mut self, Inode: Inode_type) -> &mut Self {
        self.Inode = Inode;
        self
    }

    pub fn Set_type(&mut self, Type: Type_type) -> &mut Self {
        self.Type = Type;
        self
    }

    pub fn Set_links(&mut self, Links: u64) -> &mut Self {
        self.Links = Links;
        self
    }

    pub fn Set_size(&mut self, Size: Size_type) -> &mut Self {
        self.Size = Size;
        self
    }

    pub fn Set_last_access(&mut self, Last_access: Time_type) -> &mut Self {
        self.Last_access = Last_access;
        self
    }

    pub fn Set_last_modification(&mut self, Last_modification: Time_type) -> &mut Self {
        self.Last_modification = Last_modification;
        self
    }

    pub fn Set_last_status_change(&mut self, Last_status_change: Time_type) -> &mut Self {
        self.Last_status_change = Last_status_change;
        self
    }

    pub fn Set_permissions(&mut self, Permissions: Permissions_type) -> &mut Self {
        self.Permissions = Permissions;
        self
    }

    pub fn Set_user(&mut self, User: User_identifier_type) -> &mut Self {
        self.User = User;
        self
    }

    pub fn Set_group(&mut self, Group: Group_identifier_type) -> &mut Self {
        self.Group = Group;
        self
    }
}
