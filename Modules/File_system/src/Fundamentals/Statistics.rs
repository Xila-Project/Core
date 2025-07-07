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
    file_system: File_system_identifier_type,
    inode: Inode_type,
    links: u64,
    size: Size_type,
    last_access: Time_type,
    last_modification: Time_type,
    last_status_change: Time_type,
    Type: Type_type,
    permissions: Permissions_type,
    user: User_identifier_type,
    group: Group_identifier_type,
}

impl Statistics_type {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        file_system: File_system_identifier_type,
        inode: Inode_type,
        links: u64,
        size: Size_type,
        last_access: Time_type,
        last_modification: Time_type,
        last_status_change: Time_type,
        type_value: Type_type,
        permissions: Permissions_type,
        user: User_identifier_type,
        group: Group_identifier_type,
    ) -> Self {
        Statistics_type {
            file_system,
            inode,
            links,
            size,
            last_access,
            last_modification,
            last_status_change,
            Type: type_value,
            permissions,
            user,
            group,
        }
    }

    pub const fn Get_file_system(&self) -> File_system_identifier_type {
        self.file_system
    }

    pub const fn Get_inode(&self) -> Inode_type {
        self.inode
    }

    pub const fn Get_links(&self) -> u64 {
        self.links
    }

    pub const fn Get_size(&self) -> Size_type {
        self.size
    }

    pub const fn Get_last_access(&self) -> Time_type {
        self.last_access
    }

    pub const fn Get_last_modification(&self) -> Time_type {
        self.last_modification
    }

    pub const fn Get_last_status_change(&self) -> Time_type {
        self.last_status_change
    }

    pub const fn Get_type(&self) -> Type_type {
        self.Type
    }

    pub const fn Get_permissions(&self) -> Permissions_type {
        self.permissions
    }

    pub const fn Get_user(&self) -> User_identifier_type {
        self.user
    }

    pub const fn Get_group(&self) -> Group_identifier_type {
        self.group
    }

    pub fn Set_file_system(&mut self, File_system: File_system_identifier_type) -> &mut Self {
        self.file_system = File_system;
        self
    }

    pub fn Set_inode(&mut self, Inode: Inode_type) -> &mut Self {
        self.inode = Inode;
        self
    }

    pub fn Set_type(&mut self, Type: Type_type) -> &mut Self {
        self.Type = Type;
        self
    }

    pub fn Set_links(&mut self, Links: u64) -> &mut Self {
        self.links = Links;
        self
    }

    pub fn Set_size(&mut self, Size: Size_type) -> &mut Self {
        self.size = Size;
        self
    }

    pub fn Set_last_access(&mut self, Last_access: Time_type) -> &mut Self {
        self.last_access = Last_access;
        self
    }

    pub fn Set_last_modification(&mut self, Last_modification: Time_type) -> &mut Self {
        self.last_modification = Last_modification;
        self
    }

    pub fn Set_last_status_change(&mut self, Last_status_change: Time_type) -> &mut Self {
        self.last_status_change = Last_status_change;
        self
    }

    pub fn Set_permissions(&mut self, Permissions: Permissions_type) -> &mut Self {
        self.permissions = Permissions;
        self
    }

    pub fn Set_user(&mut self, User: User_identifier_type) -> &mut Self {
        self.user = User;
        self
    }

    pub fn Set_group(&mut self, Group: Group_identifier_type) -> &mut Self {
        self.group = Group;
        self
    }
}
