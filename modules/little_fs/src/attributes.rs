use core::{ffi::c_void, mem::MaybeUninit, ptr::null_mut};

use alloc::boxed::Box;
use file_system::{Attributes, Inode, Kind, Permissions, Result, Time};
use littlefs2_sys::lfs_attr;
use users::{GroupIdentifier, UserIdentifier};

#[repr(C)] // For stable layout
pub struct InternalAttributes {
    inode: Inode,
    creation: Time,
    modification: Time,
    access: Time,
    status: Time,
    user: UserIdentifier,
    group: GroupIdentifier,
    permissions: Permissions,
    kind: Kind,
}

impl InternalAttributes {
    pub const IDENTIFIER: u8 = 1;

    pub fn new_uninitialized() -> MaybeUninit<Self> {
        MaybeUninit::<Self>::uninit()
    }

    pub fn into_lfs_attributes(self) -> *mut lfs_attr {
        let attributes = Box::into_raw(Box::new(self));

        let littlefs_attributes = Box::new(lfs_attr {
            type_: Self::IDENTIFIER,
            buffer: attributes as *mut c_void,
            size: size_of::<InternalAttributes>() as u32,
        });

        Box::into_raw(littlefs_attributes)
    }

    pub fn take_from_file_configuration(
        configuration: &mut super::littlefs::lfs_file_config,
    ) -> Option<InternalAttributes> {
        if configuration.attr_count == 0 {
            return None;
        }

        let pointer = configuration.attrs;

        if pointer.is_null() {
            return None;
        }

        let attributes = unsafe { Box::from_raw(pointer) };

        if attributes.size != size_of::<InternalAttributes>() as u32 {
            return None;
        }

        configuration.attrs = null_mut();
        configuration.attr_count = 0;

        Some(unsafe { *Box::from_raw(attributes.buffer as _) })
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
        configuration: &super::littlefs::lfs_file_config,
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

    pub fn into_attributes(&self, statistics: &mut Attributes) -> Result<()> {
        statistics
            .get_mutable_inode()
            .map(|inode| *inode = self.inode);

        statistics
            .get_mutable_access()
            .map(|time| *time = self.access);

        statistics
            .get_mutable_creation()
            .map(|time| *time = self.creation);

        statistics
            .get_mutable_modification()
            .map(|time| *time = self.modification);

        statistics
            .get_mutable_status()
            .map(|time| *time = self.status);

        statistics
            .get_mutable_permissions()
            .map(|permissions| *permissions = self.permissions);

        statistics.get_mutable_user().map(|user| *user = self.user);

        statistics
            .get_mutable_group()
            .map(|group| *group = self.group);

        statistics.get_mutable_kind().map(|kind| *kind = self.kind);

        Ok(())
    }

    pub fn from_attributes(&mut self, statistics: &Attributes) -> Result<()> {
        statistics.get_inode().map(|inode| {
            self.inode = *inode;
        });

        statistics.get_access().map(|time| {
            self.access = *time;
        });

        statistics.get_creation().map(|time| {
            self.creation = *time;
        });

        statistics.get_modification().map(|time| {
            self.modification = *time;
        });

        statistics.get_status().map(|time| {
            self.status = *time;
        });

        statistics.get_permissions().map(|permissions| {
            self.permissions = *permissions;
        });

        statistics.get_user().map(|user| {
            self.user = *user;
        });

        statistics.get_group().map(|group| {
            self.group = *group;
        });

        statistics.get_kind().map(|kind| {
            self.kind = *kind;
        });

        Ok(())
    }
}
