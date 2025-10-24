use core::fmt;

use crate::Kind;

/// Represents the permissions of a file or directory.
///
/// The permissions are divided into three groups: user, group, and others.
/// Each group has three permissions: read, write, and execute.
///
/// # Examples
///
/// ```rust
/// use file_system::{Permissions, Permission, Special};
///
/// let user = Permission::new(true, false, false); // Read only
/// let group = Permission::new(false, true, false); // Write only
/// let others = Permission::new(false, false, true); // Execute only
/// let special = Special::new(true, false, true); // Sticky and set user identifier
///
/// let permissions = Permissions::new(user, group, others, special);
///
/// assert_eq!(permissions.get_user(), user);
/// assert_eq!(permissions.get_group(), group);
/// assert_eq!(permissions.get_others(), others);
/// assert_eq!(permissions.get_special(), special);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Permissions(u16);

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let user = self.get_user();
        let group = self.get_group();
        let others = self.get_others();

        write!(f, "{user}{group}{others}")
    }
}

impl Permissions {
    pub const NONE: Self = Self::new(
        Permission::NONE,
        Permission::NONE,
        Permission::NONE,
        Special::NONE,
    );
    pub const ALL_FULL: Self = Self::new(
        Permission::FULL,
        Permission::FULL,
        Permission::FULL,
        Special::NONE,
    );
    pub const ALL_READ_WRITE: Self = Self::new(
        Permission::READ_WRITE,
        Permission::READ_WRITE,
        Permission::READ_WRITE,
        Special::NONE,
    );
    pub const USER_FULL: Self = Self::new(
        Permission::FULL,
        Permission::NONE,
        Permission::NONE,
        Special::NONE,
    );
    pub const USER_READ_WRITE: Self = Self::new(
        Permission::READ_WRITE,
        Permission::NONE,
        Permission::NONE,
        Special::NONE,
    );
    pub const EXECUTABLE: Self = Self::new(
        Permission::FULL,
        Permission::READ_EXECUTE,
        Permission::READ_EXECUTE,
        Special::NONE,
    );

    /// Creates a new permission.
    pub const fn new(
        user: Permission,
        group: Permission,
        others: Permission,
        special: Special,
    ) -> Self {
        Self(
            (special.to_unix() as u16) << 9
                | (user.to_unix() as u16) << 6
                | (group.to_unix() as u16) << 3
                | others.to_unix() as u16,
        )
    }

    /// Creates a new permission with read access for user. No access for group and others.
    pub const fn new_default(r#type: Kind) -> Self {
        match r#type {
            Kind::Directory => Self::new(
                Permission::FULL,
                Permission::READ_EXECUTE,
                Permission::READ_EXECUTE,
                Special::NONE,
            ),
            Kind::File => Self::new(
                Permission::READ_WRITE,
                Permission::READ_ONLY,
                Permission::READ_ONLY,
                Special::NONE,
            ),
            Kind::Pipe => Self::new(
                Permission::READ_WRITE,
                Permission::NONE,
                Permission::NONE,
                Special::NONE,
            ),
            Kind::BlockDevice => Self::new(
                Permission::FULL,
                Permission::READ_WRITE,
                Permission::READ_WRITE,
                Special::NONE,
            ),
            Kind::CharacterDevice => Self::new(
                Permission::READ_WRITE,
                Permission::READ_WRITE,
                Permission::NONE,
                Special::NONE,
            ),
            Kind::Socket => Self::ALL_READ_WRITE,
            Kind::SymbolicLink => Self::ALL_FULL,
        }
    }

    /// Sets the permission for the user.
    pub fn set_user(mut self, user: Permission) -> Self {
        self.0 = (self.0 & 0o7077) | (user.to_unix() as u16) << 6;
        self
    }

    /// Sets the permission for the group.
    pub fn set_group(mut self, group: Permission) -> Self {
        self.0 = (self.0 & 0o7707) | (group.to_unix() as u16) << 3;
        self
    }

    /// Sets the permission for others.
    pub fn set_others(mut self, others: Permission) -> Self {
        self.0 = (self.0 & 0o7770) | others.to_unix() as u16;
        self
    }

    /// Sets the special permissions.
    pub fn set_special(mut self, special: Special) -> Self {
        self.0 = (self.0 & 0o0777) | (special.to_unix() as u16) << 9;
        self
    }

    /// Gets the permission for the user.
    pub fn get_user(&self) -> Permission {
        Permission::from_unix(((self.0 >> 6) & 0b111) as u8).unwrap()
    }

    /// Gets the permission for the group.
    pub fn get_group(&self) -> Permission {
        Permission::from_unix(((self.0 >> 3) & 0b111) as u8).unwrap()
    }

    /// Gets the permission for others.
    pub fn get_others(&self) -> Permission {
        Permission::from_unix((self.0 & 0b111) as u8).unwrap()
    }

    /// Gets the special permissions.
    pub fn get_special(&self) -> Special {
        Special::from_unix((self.0 >> 9) as u8).unwrap()
    }

    /// Converts the permission to a Unix permission.
    pub const fn from_octal(unix: u16) -> Option<Self> {
        if unix > 0o777 {
            return None;
        }

        Some(Self(unix))
    }

    /// Converts the permission to a Unix permission.
    pub const fn as_u16(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Special(u8);

impl fmt::Display for Special {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sticky = if self.get_sticky() { "t" } else { "-" };
        let set_gid = if self.get_set_group_identifier() {
            "u"
        } else {
            "-"
        };
        let set_uid = if self.get_set_user_identifier() {
            "g"
        } else {
            "-"
        };

        write!(f, "{sticky}{set_gid}{set_uid}")
    }
}

impl Special {
    pub const NONE: Self = Self(0);
    pub const STICKY: Self = Self(1);
    pub const SET_USER_IDENTIFIER: Self = Self(2);
    pub const SET_GROUP_IDENTIFIER: Self = Self(4);

    pub fn new(sticky: bool, set_gid: bool, set_uid: bool) -> Self {
        Self((sticky as u8) | (set_gid as u8) << 1 | (set_uid as u8) << 2)
    }

    pub fn set_sticky(mut self, sticky: bool) -> Self {
        self.0 = (self.0 & 0b110) | sticky as u8;
        self
    }

    pub fn set_set_group_identifier(mut self, set_gid: bool) -> Self {
        self.0 = (self.0 & 0b101) | (set_gid as u8) << 1;
        self
    }

    pub fn set_set_user_identifier(mut self, set_uid: bool) -> Self {
        self.0 = (self.0 & 0b011) | (set_uid as u8) << 2;
        self
    }

    pub const fn get_sticky(&self) -> bool {
        self.0 & 0b001 != 0
    }

    pub const fn get_set_group_identifier(&self) -> bool {
        self.0 & 0b010 != 0
    }

    pub const fn get_set_user_identifier(&self) -> bool {
        self.0 & 0b100 != 0
    }

    pub const fn to_unix(&self) -> u8 {
        self.0
    }

    pub fn from_unix(unix: u8) -> Option<Self> {
        if unix > 0b111 {
            return None;
        }

        Some(Self(unix))
    }
}

/// Represents a permission.
///
/// The permission can be read, write, and execute.
///
/// # Examples
///
/// ```rust
/// use file_system::Permission;
///
/// let read = Permission::READ_ONLY;
/// let write = Permission::WRITE_ONLY;
/// let execute = Permission::EXECUTE_ONLY;
///
/// assert!(read.get_read() && !read.get_write() && !read.get_execute());
/// assert!(!write.get_read() && write.get_write() && !write.get_execute());
/// assert!(!execute.get_read() && !execute.get_write() && execute.get_execute());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Permission(u8);

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let read = if self.get_read() { "r" } else { "-" };
        let write = if self.get_write() { "w" } else { "-" };
        let execute = if self.get_execute() { "x" } else { "-" };

        write!(f, "{read}{write}{execute}")
    }
}

impl Permission {
    pub const READ_ONLY: Self = Self::new(true, false, false);
    pub const WRITE_ONLY: Self = Self::new(false, true, false);
    pub const EXECUTE_ONLY: Self = Self::new(false, false, true);

    pub const READ_WRITE: Self = Self::new(true, true, false);
    pub const WRITE_EXECUTE: Self = Self::new(false, true, true);
    pub const READ_EXECUTE: Self = Self::new(true, false, true);

    pub const NONE: Self = Self::new(false, false, false);
    pub const FULL: Self = Self::new(true, true, true);

    /// Creates a new permission.
    pub const fn new(read: bool, write: bool, execute: bool) -> Self {
        Self((read as u8) << 2 | (write as u8) << 1 | execute as u8)
    }

    /// Sets the read permission.
    pub fn set_read(mut self, read: bool) -> Self {
        self.0 = (self.0 & 0b011) | (read as u8) << 2;
        self
    }

    /// Sets the write permission.
    pub fn set_write(mut self, write: bool) -> Self {
        self.0 = (self.0 & 0b101) | (write as u8) << 1;
        self
    }

    /// Sets the execute permission.
    pub fn set_execute(mut self, execute: bool) -> Self {
        self.0 = (self.0 & 0b110) | execute as u8;
        self
    }

    /// Gets the read permission.
    pub const fn get_read(&self) -> bool {
        self.0 & 0b100 != 0
    }

    /// Gets the write permission.
    pub const fn get_write(&self) -> bool {
        self.0 & 0b010 != 0
    }

    /// Gets the execute permission.
    pub const fn get_execute(&self) -> bool {
        self.0 & 0b001 != 0
    }

    /// Converts the permission to a Unix permission.
    pub const fn to_unix(&self) -> u8 {
        self.0
    }

    pub fn include(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Creates a permission from a Unix permission.
    pub fn from_unix(unix: u8) -> Option<Self> {
        if unix > 0b111 {
            return None;
        }

        Some(Self(unix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_permissions() {
        let user = Permission::new(true, false, false); // Read only
        let group = Permission::new(false, true, false); // Write only
        let others = Permission::new(false, false, true); // Execute only
        let special = Special::new(true, false, true); // Sticky and set user identifier
        let permissions = Permissions::new(user, group, others, special);
        assert_eq!(permissions.0, 0b101_100_010_001);
    }

    #[test]
    fn test_new_permission() {
        assert_eq!(Permission::READ_ONLY.0, 0b100);
        assert_eq!(Permission::WRITE_ONLY.0, 0b010);
        assert_eq!(Permission::EXECUTE_ONLY.0, 0b001);
        assert_eq!(Permission::READ_WRITE.0, 0b110);
        assert_eq!(Permission::WRITE_EXECUTE.0, 0b011);
        assert_eq!(Permission::READ_EXECUTE.0, 0b101);
        assert_eq!(Permission::NONE.0, 0b000);
        assert_eq!(Permission::FULL.0, 0b111);
    }

    #[test]
    fn test_permission_type_to_unix() {
        let read = Permission::READ_ONLY;
        assert_eq!(read.to_unix(), 4);
        let write = Permission::WRITE_ONLY;
        assert_eq!(write.to_unix(), 2);
        let execute = Permission::EXECUTE_ONLY;
        assert_eq!(execute.to_unix(), 1);
        let full = Permission::FULL;
        assert_eq!(full.to_unix(), 7);
        let none = Permission::NONE;
        assert_eq!(none.to_unix(), 0);
    }

    #[test]
    fn test_permission_type_from_unix() {
        let read = Permission::from_unix(4).unwrap();
        assert!(read.get_read() && !read.get_write() && !read.get_execute());
        let write = Permission::from_unix(2).unwrap();
        assert!(!write.get_read() && write.get_write() && !write.get_execute());
        let execute = Permission::from_unix(1).unwrap();
        assert!(!execute.get_read() && !execute.get_write() && execute.get_execute());
        let full = Permission::from_unix(7).unwrap();
        assert!(full.get_read() && full.get_write() && full.get_execute());
        let no = Permission::from_unix(0).unwrap();
        assert!(!no.get_read() && !no.get_write() && !no.get_execute());
    }

    #[test]
    fn test_permissions_type_from_unix() {
        let permissions = Permissions::from_octal(0b101_101_101).unwrap();
        assert_eq!(permissions.get_user().to_unix(), 5);
        assert_eq!(permissions.get_group().to_unix(), 5);
        assert_eq!(permissions.get_others().to_unix(), 5);
    }

    #[test]
    fn test_permissions_type_to_unix() {
        let user = Permission::new(true, false, true); // Read and execute
        let group = Permission::new(true, true, false); // Read and write
        let others = Permission::new(false, true, true); // Write and execute
        let special = Special::new(true, false, true); // Sticky and set user identifier
        let permissions = Permissions::new(user, group, others, special);
        assert_eq!(permissions.as_u16(), 0b101_101_110_011);
    }

    #[test]
    fn test_permission_type_include() {
        let read = Permission::READ_ONLY;
        let write = Permission::WRITE_ONLY;
        let read_write = Permission::READ_WRITE;
        let read_execute = Permission::READ_EXECUTE;
        let write_execute = Permission::WRITE_EXECUTE;
        let execute = Permission::EXECUTE_ONLY;
        let full = Permission::FULL;
        let no = Permission::NONE;

        assert!(full.include(read));
        assert!(full.include(write));
        assert!(full.include(execute));
        assert!(full.include(read_write));
        assert!(full.include(read_execute));
        assert!(full.include(write_execute));
        assert!(full.include(full));
        assert!(full.include(no));

        assert!(read.include(read));
        assert!(!read.include(write));
        assert!(!read.include(execute));
        assert!(!read.include(read_write));
        assert!(!read.include(read_execute));
        assert!(!read.include(write_execute));
        assert!(!read.include(full));
        assert!(read.include(no));

        assert!(!write.include(read));
        assert!(write.include(write));
        assert!(!write.include(execute));
        assert!(!write.include(read_write));
        assert!(!write.include(read_execute));
        assert!(!write.include(write_execute));
        assert!(!write.include(full));
        assert!(write.include(no));

        assert!(!execute.include(read));
        assert!(!execute.include(write));
        assert!(execute.include(execute));
        assert!(!execute.include(read_write));
        assert!(!execute.include(read_execute));
        assert!(!execute.include(write_execute));
        assert!(!execute.include(full));
        assert!(execute.include(no));

        assert!(read_write.include(read));
        assert!(read_write.include(write));
        assert!(!read_write.include(execute));
        assert!(read_write.include(read_write));
        assert!(!read_write.include(read_execute));
        assert!(!read_write.include(write_execute));
        assert!(!read_write.include(full));
        assert!(read_write.include(no));

        assert!(read_execute.include(read));
        assert!(!read_execute.include(write));
        assert!(read_execute.include(execute));
        assert!(!read_execute.include(read_write));
        assert!(read_execute.include(read_execute));
        assert!(!read_execute.include(write_execute));
        assert!(!read_execute.include(full));
        assert!(read_execute.include(no));

        assert!(!write_execute.include(read));
        assert!(write_execute.include(write));
        assert!(write_execute.include(execute));
        assert!(!write_execute.include(read_write));
        assert!(!write_execute.include(read_execute));
        assert!(write_execute.include(write_execute));
        assert!(!write_execute.include(full));
        assert!(write_execute.include(no));

        assert!(!no.include(read));
        assert!(!no.include(write));
        assert!(!no.include(execute));
        assert!(!no.include(read_write));
        assert!(!no.include(read_execute));
        assert!(!no.include(write_execute));
        assert!(!no.include(full));
        assert!(no.include(no));
    }
}
