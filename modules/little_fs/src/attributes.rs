use core::{ffi::c_void, fmt::Debug, mem::MaybeUninit};

use alloc::boxed::Box;
use file_system::{Attributes, Inode, Kind, Permissions, Result, Time};
use littlefs2_sys::lfs_attr;
use users::{GroupIdentifier, UserIdentifier};

#[repr(C)] // For stable layout
pub struct InternalAttributes {
    pub inode: Inode,
    pub creation: Time,
    pub modification: Time,
    pub access: Time,
    pub status: Time,
    pub user: UserIdentifier,
    pub group: GroupIdentifier,
    pub permissions: Permissions,
    pub kind: Kind,
}

impl Debug for InternalAttributes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InternalAttributes")
            .field("inode", &self.inode)
            //.field("creation", &self.creation)
            //.field("modification", &self.modification)
            //.field("access", &self.access)
            //.field("status", &self.status)
            .field("user", &self.user)
            .field("group", &self.group)
            .field("permissions", &self.permissions)
            .field("kind", &(self.kind as u8))
            .finish()
    }
}

impl InternalAttributes {
    pub const IDENTIFIER: u8 = 1;

    pub fn new_uninitialized() -> MaybeUninit<Self> {
        MaybeUninit::<Self>::uninit()
    }

    pub fn into_lfs_attributes(self) -> *mut lfs_attr {
        let attributes = Box::leak(Box::new(self));

        let littlefs_attributes = Box::new(lfs_attr {
            type_: Self::IDENTIFIER,
            buffer: attributes as *mut _ as *mut c_void,
            size: size_of::<InternalAttributes>() as u32,
        });

        Box::leak(littlefs_attributes)
    }

    pub fn get_from_file_configuration(
        configuration: &super::littlefs::lfs_file_config,
    ) -> Option<&InternalAttributes> {
        if configuration.attr_count == 0 {
            return None;
        }

        let pointer = configuration.attrs;

        if pointer.is_null() {
            return None;
        }

        let littlefs_attributes = unsafe { Box::leak(Box::from_raw(pointer)) };

        if littlefs_attributes.size != size_of::<InternalAttributes>() as u32 {
            return None;
        }

        let attributes = Box::leak(unsafe { Box::from_raw(littlefs_attributes.buffer as _) });

        Some(attributes)
    }

    pub fn get_mutable_from_file_configuration(
        configuration: &mut super::littlefs::lfs_file_config,
    ) -> Option<&mut InternalAttributes> {
        if configuration.attr_count == 0 {
            return None;
        }

        let pointer = configuration.attrs;

        if pointer.is_null() {
            return None;
        }

        let littlefs_attributes = unsafe { Box::leak(Box::from_raw(pointer)) };

        if littlefs_attributes.size != size_of::<InternalAttributes>() as u32 {
            return None;
        }

        let attributes = Box::leak(unsafe { Box::from_raw(littlefs_attributes.buffer as _) });

        Some(attributes)
    }

    pub fn update_attributes(&self, statistics: &mut Attributes) -> Result<()> {
        if let Some(inode) = statistics.get_mutable_inode() {
            *inode = self.inode;
        }

        if let Some(time) = statistics.get_mutable_access() {
            *time = self.access;
        }

        if let Some(time) = statistics.get_mutable_creation() {
            *time = self.creation;
        }

        if let Some(time) = statistics.get_mutable_modification() {
            *time = self.modification;
        }

        if let Some(time) = statistics.get_mutable_status() {
            *time = self.status;
        }

        if let Some(permissions) = statistics.get_mutable_permissions() {
            *permissions = self.permissions;
        }

        if let Some(user) = statistics.get_mutable_user() {
            *user = self.user;
        }

        if let Some(group) = statistics.get_mutable_group() {
            *group = self.group;
        }

        if let Some(kind) = statistics.get_mutable_kind() {
            *kind = self.kind;
        }

        Ok(())
    }

    pub fn update_with_attributes(&mut self, statistics: &Attributes) -> Result<()> {
        if let Some(inode) = statistics.get_inode() {
            self.inode = *inode;
        }

        if let Some(time) = statistics.get_access() {
            self.access = *time;
        }

        if let Some(time) = statistics.get_creation() {
            self.creation = *time;
        }

        if let Some(time) = statistics.get_modification() {
            self.modification = *time;
        }

        if let Some(time) = statistics.get_status() {
            self.status = *time;
        }

        if let Some(permissions) = statistics.get_permissions() {
            self.permissions = *permissions;
        }

        if let Some(user) = statistics.get_user() {
            self.user = *user;
        }

        if let Some(group) = statistics.get_group() {
            self.group = *group;
        }

        if let Some(kind) = statistics.get_kind() {
            self.kind = *kind;
        }

        Ok(())
    }
}
