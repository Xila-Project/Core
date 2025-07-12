use users::{GroupIdentifier, UserIdentifier};

use crate::{FileSystemIdentifier, Kind, Size, Time};

use super::{Inode, Permissions};

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
    file_system: FileSystemIdentifier,
    group: GroupIdentifier,
    inode: Inode,
    last_access: Time,
    last_modification: Time,
    last_status_change: Time,
    links: u64,
    permissions: Permissions,
    r#type: Kind,
    size: Size,
    user: UserIdentifier,
}

impl Statistics_type {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        file_system: FileSystemIdentifier,
        inode: Inode,
        links: u64,
        size: Size,
        last_access: Time,
        last_modification: Time,
        last_status_change: Time,
        type_value: Kind,
        permissions: Permissions,
        user: UserIdentifier,
        group: GroupIdentifier,
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

    pub const fn get_file_system(&self) -> FileSystemIdentifier {
        self.file_system
    }

    pub const fn get_inode(&self) -> Inode {
        self.inode
    }

    pub const fn get_links(&self) -> u64 {
        self.links
    }

    pub const fn get_size(&self) -> Size {
        self.size
    }

    pub const fn get_last_access(&self) -> Time {
        self.last_access
    }

    pub const fn get_last_modification(&self) -> Time {
        self.last_modification
    }

    pub const fn get_last_status_change(&self) -> Time {
        self.last_status_change
    }

    pub const fn get_type(&self) -> Kind {
        self.r#type
    }

    pub const fn get_permissions(&self) -> Permissions {
        self.permissions
    }

    pub const fn get_user(&self) -> UserIdentifier {
        self.user
    }

    pub const fn get_group(&self) -> GroupIdentifier {
        self.group
    }

    pub fn set_file_system(&mut self, file_system: FileSystemIdentifier) -> &mut Self {
        self.file_system = file_system;
        self
    }

    pub fn set_inode(&mut self, inode: Inode) -> &mut Self {
        self.inode = inode;
        self
    }

    pub fn set_type(&mut self, r#type: Kind) -> &mut Self {
        self.r#type = r#type;
        self
    }

    pub fn set_links(&mut self, links: u64) -> &mut Self {
        self.links = links;
        self
    }

    pub fn set_size(&mut self, size: Size) -> &mut Self {
        self.size = size;
        self
    }

    pub fn set_last_access(&mut self, last_access: Time) -> &mut Self {
        self.last_access = last_access;
        self
    }

    pub fn set_last_modification(&mut self, last_modification: Time) -> &mut Self {
        self.last_modification = last_modification;
        self
    }

    pub fn set_last_status_change(&mut self, last_status_change: Time) -> &mut Self {
        self.last_status_change = last_status_change;
        self
    }

    pub fn set_permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.permissions = permissions;
        self
    }

    pub fn set_user(&mut self, user: UserIdentifier) -> &mut Self {
        self.user = user;
        self
    }

    pub fn set_group(&mut self, group: GroupIdentifier) -> &mut Self {
        self.group = group;
        self
    }
}
