use core::ops::BitOr;

use crate::{Inode, Kind, Permissions, Size, Time};
use users::{GroupIdentifier, UserIdentifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct AttributesMask(u16);

impl AttributesMask {
    pub const INODE: Self = Self(1 << 0);
    pub const KIND: Self = Self(1 << 1);
    pub const SIZE: Self = Self(1 << 2);
    pub const LINKS: Self = Self(1 << 3);
    pub const CREATION_TIME: Self = Self(1 << 4);
    pub const MODIFICATION_TIME: Self = Self(1 << 5);
    pub const ACCESS_TIME: Self = Self(1 << 6);
    pub const STATUS_TIME: Self = Self(1 << 7);
    pub const PERMISSIONS: Self = Self(1 << 8);
    pub const USER: Self = Self(1 << 9);
    pub const GROUP: Self = Self(1 << 10);
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(0b111_1111_1111);

    pub fn contains(&self, other: AttributesMask) -> bool {
        ((*self).0 & other.0) == other.0
    }

    pub fn negate(&self) -> u16 {
        !((*self).0)
    }

    pub fn are_all_set(&self) -> bool {
        (*self) == Self::ALL
    }

    pub fn set(self, other: AttributesMask) -> Self {
        Self(self.0 | other.0)
    }

    pub fn unset(self, other: AttributesMask) -> Self {
        Self(self.0 & !other.0)
    }
}

impl BitOr for AttributesMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// File attributes.
///
/// The attributes are metadata associated with the file that stores:
/// - The file type.
/// - The file creation time.
/// - The file modification time.
/// - The file access time.
/// - The file permissions.
/// - The file owner.
/// - The file group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attributes {
    /// The file inode.
    inode: Inode,
    /// The file type.
    kind: Kind,
    /// Size
    size: Size,
    /// Links
    links: Size,
    /// The file status change time.
    status: Time,
    /// The file modification time.
    modification: Time,
    /// The file access time.
    access: Time,
    /// The file creation time.
    creation: Time,
    /// The file permissions.
    permissions: Permissions,
    /// The file owner.
    user: UserIdentifier,
    /// The file group.
    group: GroupIdentifier,
    /// Mask
    mask: AttributesMask,
}

/// Macro to generate getter, mutable getter, and setter methods for Attributes fields
macro_rules! generate_attribute_accessors {
    ($getter:ident, $mutable_getter:ident, $setter:ident, $field:ident, $field_type:ty, $mask_flag:expr) => {
        pub fn $getter(&self) -> Option<&$field_type> {
            Self::get(&self.$field, $mask_flag)
        }

        pub fn $mutable_getter(&mut self) -> Option<&mut $field_type> {
            Self::get_mutable(&mut self.$field, $mask_flag)
        }

        pub fn $setter(mut self, $field: $field_type) -> Self {
            self.mask = self.mask.set($mask_flag);
            self.$field = $field;

            self
        }
    };
}

impl Attributes {
    pub fn new() -> Self {
        Attributes {
            inode: 0,
            kind: Kind::File,
            size: 0,
            links: 0,
            creation: Time::new(0),
            status: Time::new(0),
            modification: Time::new(0),
            access: Time::new(0),
            permissions: Permissions::NONE,
            user: UserIdentifier::ROOT,
            group: GroupIdentifier::ROOT,
            mask: AttributesMask::NONE,
        }
    }

    fn get<T>(field: &T, mask: AttributesMask) -> Option<&T> {
        if mask.contains(mask) {
            Some(field)
        } else {
            None
        }
    }

    fn get_mutable<T>(field: &mut T, mask: AttributesMask) -> Option<&mut T> {
        if mask.contains(mask) {
            Some(field)
        } else {
            None
        }
    }

    pub fn get_mask(&self) -> AttributesMask {
        self.mask
    }

    pub fn set_mask(mut self, mask: AttributesMask) -> Self {
        self.mask = mask;
        self
    }

    generate_attribute_accessors!(
        get_inode,
        get_mutable_inode,
        set_inode,
        inode,
        Inode,
        AttributesMask::INODE
    );
    generate_attribute_accessors!(
        get_size,
        get_mutable_size,
        set_size,
        size,
        Size,
        AttributesMask::SIZE
    );
    generate_attribute_accessors!(
        get_kind,
        get_mutable_kind,
        set_kind,
        kind,
        Kind,
        AttributesMask::KIND
    );
    generate_attribute_accessors!(
        get_permissions,
        get_mutable_permissions,
        set_permissions,
        permissions,
        Permissions,
        AttributesMask::PERMISSIONS
    );
    generate_attribute_accessors!(
        get_user,
        get_mutable_user,
        set_user,
        user,
        UserIdentifier,
        AttributesMask::USER
    );
    generate_attribute_accessors!(
        get_group,
        get_mutable_group,
        set_group,
        group,
        GroupIdentifier,
        AttributesMask::GROUP
    );
    generate_attribute_accessors!(
        get_creation,
        get_mutable_creation,
        set_creation,
        creation,
        Time,
        AttributesMask::CREATION_TIME
    );
    generate_attribute_accessors!(
        get_modification,
        get_mutable_modification,
        set_modification,
        modification,
        Time,
        AttributesMask::MODIFICATION_TIME
    );
    generate_attribute_accessors!(
        get_access,
        get_mutable_access,
        set_access,
        access,
        Time,
        AttributesMask::ACCESS_TIME
    );
    generate_attribute_accessors!(
        get_status,
        get_mutable_status,
        set_status,
        status,
        Time,
        AttributesMask::STATUS_TIME
    );
    generate_attribute_accessors!(
        get_links,
        get_mutable_links,
        set_links,
        links,
        Size,
        AttributesMask::LINKS
    );
}
