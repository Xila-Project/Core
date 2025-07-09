use users::{Group_identifier_type, User_identifier_type};

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
    group: Group_identifier_type,
    inode: Inode_type,
    last_access: Time_type,
    last_modification: Time_type,
    last_status_change: Time_type,
    links: u64,
    permissions: Permissions_type,
    r#type: Type_type,
    size: Size_type,
    user: User_identifier_type,
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
            r#type: type_value,
            permissions,
            user,
            group,
        }
    }

    pub const fn get_file_system(&self) -> File_system_identifier_type {
        self.file_system
    }

    pub const fn get_inode(&self) -> Inode_type {
        self.inode
    }

    pub const fn get_links(&self) -> u64 {
        self.links
    }

    pub const fn get_size(&self) -> Size_type {
        self.size
    }

    pub const fn get_last_access(&self) -> Time_type {
        self.last_access
    }

    pub const fn get_last_modification(&self) -> Time_type {
        self.last_modification
    }

    pub const fn get_last_status_change(&self) -> Time_type {
        self.last_status_change
    }

    pub const fn get_type(&self) -> Type_type {
        self.r#type
    }

    pub const fn get_permissions(&self) -> Permissions_type {
        self.permissions
    }

    pub const fn get_user(&self) -> User_identifier_type {
        self.user
    }

    pub const fn get_group(&self) -> Group_identifier_type {
        self.group
    }

    pub fn set_file_system(&mut self, file_system: File_system_identifier_type) -> &mut Self {
        self.file_system = file_system;
        self
    }

    pub fn set_inode(&mut self, inode: Inode_type) -> &mut Self {
        self.inode = inode;
        self
    }

    pub fn set_type(&mut self, r#type: Type_type) -> &mut Self {
        self.r#type = r#type;
        self
    }

    pub fn set_links(&mut self, links: u64) -> &mut Self {
        self.links = links;
        self
    }

    pub fn set_size(&mut self, size: Size_type) -> &mut Self {
        self.size = size;
        self
    }

    pub fn set_last_access(&mut self, last_access: Time_type) -> &mut Self {
        self.last_access = last_access;
        self
    }

    pub fn set_last_modification(&mut self, last_modification: Time_type) -> &mut Self {
        self.last_modification = last_modification;
        self
    }

    pub fn set_last_status_change(&mut self, last_status_change: Time_type) -> &mut Self {
        self.last_status_change = last_status_change;
        self
    }

    pub fn set_permissions(&mut self, permissions: Permissions_type) -> &mut Self {
        self.permissions = permissions;
        self
    }

    pub fn set_user(&mut self, user: User_identifier_type) -> &mut Self {
        self.user = user;
        self
    }

    pub fn set_group(&mut self, group: Group_identifier_type) -> &mut Self {
        self.group = group;
        self
    }
}
