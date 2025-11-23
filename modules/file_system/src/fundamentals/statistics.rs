use users::{GroupIdentifier, UserIdentifier};

use crate::{Attributes, Kind, Size, Time};

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
pub struct Statistics {
    pub group: GroupIdentifier,
    pub inode: Inode,
    pub creation: Time,
    pub access: Time,
    pub modification: Time,
    pub status: Time,
    pub links: Size,
    pub permissions: Permissions,
    pub kind: Kind,
    pub size: Size,
    pub user: UserIdentifier,
}

impl Statistics {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        inode: Inode,
        links: Size,
        size: Size,
        creation: Time,
        access: Time,
        modification: Time,
        status: Time,
        type_value: Kind,
        permissions: Permissions,
        user: UserIdentifier,
        group: GroupIdentifier,
    ) -> Self {
        Statistics {
            inode,
            links,
            size,
            creation,
            access,
            modification,
            status,
            kind: type_value,
            permissions,
            user,
            group,
        }
    }

    pub fn from_attributes(attributes: &Attributes) -> Option<Self> {
        Some(Statistics::new(
            *attributes.get_inode()?,
            *attributes.get_links()?,
            *attributes.get_size()?,
            *attributes.get_creation()?,
            *attributes.get_access()?,
            *attributes.get_modification()?,
            *attributes.get_status()?,
            *attributes.get_kind()?,
            *attributes.get_permissions()?,
            *attributes.get_user()?,
            *attributes.get_group()?,
        ))
    }
}
