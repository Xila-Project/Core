use core::fmt;

use crate::Type_type;

/// Represents the permissions of a file or directory.
///
/// The permissions are divided into three groups: user, group, and others.
/// Each group has three permissions: read, write, and execute.
///
/// # Examples
///
/// ```rust
/// use File_system::{Permissions_type, Permission_type, Special_type};
///
/// let user = Permission_type::New(true, false, false); // Read only
/// let group = Permission_type::New(false, true, false); // Write only
/// let others = Permission_type::New(false, false, true); // Execute only
/// let special = Special_type::New(true, false, true); // Sticky and set user identifier
///
/// let permissions = Permissions_type::New(user, group, others, special);
///
/// assert_eq!(permissions.Get_user(), user);
/// assert_eq!(permissions.Get_group(), group);
/// assert_eq!(permissions.Get_others(), others);
/// assert_eq!(permissions.Get_special(), special);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Permissions_type(u16);

impl fmt::Display for Permissions_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let User = self.Get_user();
        let Group = self.Get_group();
        let Others = self.Get_others();

        write!(f, "{}{}{}", User, Group, Others)
    }
}

impl Permissions_type {
    pub const None: Self = Self::New(
        Permission_type::None,
        Permission_type::None,
        Permission_type::None,
        Special_type::None,
    );
    pub const All_full: Self = Self::New(
        Permission_type::Full,
        Permission_type::Full,
        Permission_type::Full,
        Special_type::None,
    );
    pub const All_read_write: Self = Self::New(
        Permission_type::Read_write,
        Permission_type::Read_write,
        Permission_type::Read_write,
        Special_type::None,
    );
    pub const User_full: Self = Self::New(
        Permission_type::Full,
        Permission_type::None,
        Permission_type::None,
        Special_type::None,
    );
    pub const User_read_write: Self = Self::New(
        Permission_type::Read_write,
        Permission_type::None,
        Permission_type::None,
        Special_type::None,
    );
    pub const Executable: Self = Self::New(
        Permission_type::Full,
        Permission_type::Read_execute,
        Permission_type::Read_execute,
        Special_type::None,
    );

    /// Creates a new permission.
    pub const fn New(
        User: Permission_type,
        Group: Permission_type,
        Others: Permission_type,
        Special: Special_type,
    ) -> Self {
        Self(
            (Special.To_unix() as u16) << 9
                | (User.To_unix() as u16) << 6
                | (Group.To_unix() as u16) << 3
                | Others.To_unix() as u16,
        )
    }

    /// Creates a new permission with read access for user. No access for group and others.
    pub const fn New_default(Type: Type_type) -> Self {
        match Type {
            Type_type::Directory => Self::New(
                Permission_type::Full,
                Permission_type::Read_execute,
                Permission_type::Read_execute,
                Special_type::None,
            ),
            Type_type::File => Self::New(
                Permission_type::Read_write,
                Permission_type::Read_only,
                Permission_type::Read_only,
                Special_type::None,
            ),
            Type_type::Pipe => Self::New(
                Permission_type::Read_write,
                Permission_type::None,
                Permission_type::None,
                Special_type::None,
            ),
            Type_type::Block_device => Self::New(
                Permission_type::Full,
                Permission_type::Read_write,
                Permission_type::Read_write,
                Special_type::None,
            ),
            Type_type::Character_device => Self::New(
                Permission_type::Read_write,
                Permission_type::Read_write,
                Permission_type::None,
                Special_type::None,
            ),
            Type_type::Socket => Self::All_read_write,
            Type_type::Symbolic_link => Self::All_full,
        }
    }

    /// Sets the permission for the user.
    pub fn Set_user(mut self, User: Permission_type) -> Self {
        self.0 = (self.0 & 0o7077) | (User.To_unix() as u16) << 6;
        self
    }

    /// Sets the permission for the group.
    pub fn Set_group(mut self, Group: Permission_type) -> Self {
        self.0 = (self.0 & 0o7707) | (Group.To_unix() as u16) << 3;
        self
    }

    /// Sets the permission for others.
    pub fn Set_others(mut self, Others: Permission_type) -> Self {
        self.0 = (self.0 & 0o7770) | Others.To_unix() as u16;
        self
    }

    /// Sets the special permissions.
    pub fn Set_special(mut self, Special: Special_type) -> Self {
        self.0 = (self.0 & 0o0777) | (Special.To_unix() as u16) << 9;
        self
    }

    /// Gets the permission for the user.
    pub fn Get_user(&self) -> Permission_type {
        Permission_type::From_unix(((self.0 >> 6) & 0b111) as u8).unwrap()
    }

    /// Gets the permission for the group.
    pub fn Get_group(&self) -> Permission_type {
        Permission_type::From_unix(((self.0 >> 3) & 0b111) as u8).unwrap()
    }

    /// Gets the permission for others.
    pub fn Get_others(&self) -> Permission_type {
        Permission_type::From_unix((self.0 & 0b111) as u8).unwrap()
    }

    /// Gets the special permissions.
    pub fn Get_special(&self) -> Special_type {
        Special_type::From_unix((self.0 >> 9) as u8).unwrap()
    }

    /// Converts the permission to a Unix permission.
    pub const fn From_octal(Unix: u16) -> Option<Self> {
        if Unix > 0o777 {
            return None;
        }

        Some(Self(Unix))
    }

    /// Converts the permission to a Unix permission.
    pub const fn As_u16(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Special_type(u8);

impl fmt::Display for Special_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Sticky = if self.Get_sticky() { "t" } else { "-" };
        let Set_gid = if self.Get_set_group_identifier() {
            "u"
        } else {
            "-"
        };
        let Set_uid = if self.Get_set_user_identifier() {
            "g"
        } else {
            "-"
        };

        write!(f, "{}{}{}", Sticky, Set_gid, Set_uid)
    }
}

impl Special_type {
    pub const None: Self = Self(0);
    pub const Sticky: Self = Self(1);
    pub const Set_user_identifier: Self = Self(2);
    pub const Set_group_identifier: Self = Self(4);

    pub fn New(Sticky: bool, Set_gid: bool, Set_uid: bool) -> Self {
        Self((Sticky as u8) | (Set_gid as u8) << 1 | (Set_uid as u8) << 2)
    }

    pub fn Set_sticky(mut self, Sticky: bool) -> Self {
        self.0 = (self.0 & 0b110) | Sticky as u8;
        self
    }

    pub fn Set_set_group_identifier(mut self, Set_gid: bool) -> Self {
        self.0 = (self.0 & 0b101) | (Set_gid as u8) << 1;
        self
    }

    pub fn Set_set_user_identifier(mut self, Set_uid: bool) -> Self {
        self.0 = (self.0 & 0b011) | (Set_uid as u8) << 2;
        self
    }

    pub const fn Get_sticky(&self) -> bool {
        self.0 & 0b001 != 0
    }

    pub const fn Get_set_group_identifier(&self) -> bool {
        self.0 & 0b010 != 0
    }

    pub const fn Get_set_user_identifier(&self) -> bool {
        self.0 & 0b100 != 0
    }

    pub const fn To_unix(&self) -> u8 {
        self.0
    }

    pub fn From_unix(Unix: u8) -> Option<Self> {
        if Unix > 0b111 {
            return None;
        }

        Some(Self(Unix))
    }
}

/// Represents a permission.
///
/// The permission can be read, write, and execute.
///
/// # Examples
///
/// ```rust
/// use File_system::Permission_type;
///
/// let read = Permission_type::Read_only;
/// let write = Permission_type::Write_only;
/// let execute = Permission_type::Execute_only;
///
/// assert!(read.Get_read() && !read.Get_write() && !read.Get_execute());
/// assert!(!write.Get_read() && write.Get_write() && !write.Get_execute());
/// assert!(!execute.Get_read() && !execute.Get_write() && execute.Get_execute());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Permission_type(u8);

impl fmt::Display for Permission_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Read = if self.Get_read() { "r" } else { "-" };
        let Write = if self.Get_write() { "w" } else { "-" };
        let Execute = if self.Get_execute() { "x" } else { "-" };

        write!(f, "{}{}{}", Read, Write, Execute)
    }
}

impl Permission_type {
    pub const Read_only: Self = Self::New(true, false, false);
    pub const Write_only: Self = Self::New(false, true, false);
    pub const Execute_only: Self = Self::New(false, false, true);

    pub const Read_write: Self = Self::New(true, true, false);
    pub const Write_execute: Self = Self::New(false, true, true);
    pub const Read_execute: Self = Self::New(true, false, true);

    pub const None: Self = Self::New(false, false, false);
    pub const Full: Self = Self::New(true, true, true);

    /// Creates a new permission.
    pub const fn New(Read: bool, Write: bool, Execute: bool) -> Self {
        Self((Read as u8) << 2 | (Write as u8) << 1 | Execute as u8)
    }

    /// Sets the read permission.
    pub fn Set_read(mut self, Read: bool) -> Self {
        self.0 = (self.0 & 0b011) | (Read as u8) << 2;
        self
    }

    /// Sets the write permission.
    pub fn Set_write(mut self, Write: bool) -> Self {
        self.0 = (self.0 & 0b101) | (Write as u8) << 1;
        self
    }

    /// Sets the execute permission.
    pub fn Set_execute(mut self, Execute: bool) -> Self {
        self.0 = (self.0 & 0b110) | Execute as u8;
        self
    }

    /// Gets the read permission.
    pub const fn Get_read(&self) -> bool {
        self.0 & 0b100 != 0
    }

    /// Gets the write permission.
    pub const fn Get_write(&self) -> bool {
        self.0 & 0b010 != 0
    }

    /// Gets the execute permission.
    pub const fn Get_execute(&self) -> bool {
        self.0 & 0b001 != 0
    }

    /// Converts the permission to a Unix permission.
    pub const fn To_unix(&self) -> u8 {
        self.0
    }

    pub fn Include(&self, Other: Self) -> bool {
        (self.0 & Other.0) == Other.0
    }

    /// Creates a permission from a Unix permission.
    pub fn From_unix(Unix: u8) -> Option<Self> {
        if Unix > 0b111 {
            return None;
        }

        Some(Self(Unix))
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_new_permissions() {
        let user = Permission_type::New(true, false, false); // Read only
        let group = Permission_type::New(false, true, false); // Write only
        let others = Permission_type::New(false, false, true); // Execute only
        let special = Special_type::New(true, false, true); // Sticky and set user identifier
        let permissions = Permissions_type::New(user, group, others, special);
        assert_eq!(permissions.0, 0b101_100_010_001);
    }

    #[test]
    fn Test_new_permission() {
        assert_eq!(Permission_type::Read_only.0, 0b100);
        assert_eq!(Permission_type::Write_only.0, 0b010);
        assert_eq!(Permission_type::Execute_only.0, 0b001);
        assert_eq!(Permission_type::Read_write.0, 0b110);
        assert_eq!(Permission_type::Write_execute.0, 0b011);
        assert_eq!(Permission_type::Read_execute.0, 0b101);
        assert_eq!(Permission_type::None.0, 0b000);
        assert_eq!(Permission_type::Full.0, 0b111);
    }

    #[test]
    fn Test_permission_type_to_unix() {
        let read = Permission_type::Read_only;
        assert_eq!(read.To_unix(), 4);
        let write = Permission_type::Write_only;
        assert_eq!(write.To_unix(), 2);
        let execute = Permission_type::Execute_only;
        assert_eq!(execute.To_unix(), 1);
        let full = Permission_type::Full;
        assert_eq!(full.To_unix(), 7);
        let none = Permission_type::None;
        assert_eq!(none.To_unix(), 0);
    }

    #[test]
    fn Test_permission_type_from_unix() {
        let Read = Permission_type::From_unix(4).unwrap();
        assert!(Read.Get_read() && !Read.Get_write() && !Read.Get_execute());
        let Write = Permission_type::From_unix(2).unwrap();
        assert!(!Write.Get_read() && Write.Get_write() && !Write.Get_execute());
        let Execute = Permission_type::From_unix(1).unwrap();
        assert!(!Execute.Get_read() && !Execute.Get_write() && Execute.Get_execute());
        let Full = Permission_type::From_unix(7).unwrap();
        assert!(Full.Get_read() && Full.Get_write() && Full.Get_execute());
        let No = Permission_type::From_unix(0).unwrap();
        assert!(!No.Get_read() && !No.Get_write() && !No.Get_execute());
    }

    #[test]
    fn Test_permissions_type_from_unix() {
        let Permissions = Permissions_type::From_octal(0b101_101_101).unwrap();
        assert_eq!(Permissions.Get_user().To_unix(), 5);
        assert_eq!(Permissions.Get_group().To_unix(), 5);
        assert_eq!(Permissions.Get_others().To_unix(), 5);
    }

    #[test]
    fn Test_permissions_type_to_unix() {
        let User = Permission_type::New(true, false, true); // Read and execute
        let Group = Permission_type::New(true, true, false); // Read and write
        let Others = Permission_type::New(false, true, true); // Write and execute
        let Special = Special_type::New(true, false, true); // Sticky and set user identifier
        let Permissions = Permissions_type::New(User, Group, Others, Special);
        assert_eq!(Permissions.As_u16(), 0b101_101_110_011);
    }

    #[test]
    fn Test_permission_type_include() {
        let Read = Permission_type::Read_only;
        let Write = Permission_type::Write_only;
        let Read_write = Permission_type::Read_write;
        let Read_execute = Permission_type::Read_execute;
        let Write_execute = Permission_type::Write_execute;
        let Execute = Permission_type::Execute_only;
        let Full = Permission_type::Full;
        let No = Permission_type::None;

        assert!(Full.Include(Read));
        assert!(Full.Include(Write));
        assert!(Full.Include(Execute));
        assert!(Full.Include(Read_write));
        assert!(Full.Include(Read_execute));
        assert!(Full.Include(Write_execute));
        assert!(Full.Include(Full));
        assert!(Full.Include(No));

        assert!(Read.Include(Read));
        assert!(!Read.Include(Write));
        assert!(!Read.Include(Execute));
        assert!(!Read.Include(Read_write));
        assert!(!Read.Include(Read_execute));
        assert!(!Read.Include(Write_execute));
        assert!(!Read.Include(Full));
        assert!(Read.Include(No));

        assert!(!Write.Include(Read));
        assert!(Write.Include(Write));
        assert!(!Write.Include(Execute));
        assert!(!Write.Include(Read_write));
        assert!(!Write.Include(Read_execute));
        assert!(!Write.Include(Write_execute));
        assert!(!Write.Include(Full));
        assert!(Write.Include(No));

        assert!(!Execute.Include(Read));
        assert!(!Execute.Include(Write));
        assert!(Execute.Include(Execute));
        assert!(!Execute.Include(Read_write));
        assert!(!Execute.Include(Read_execute));
        assert!(!Execute.Include(Write_execute));
        assert!(!Execute.Include(Full));
        assert!(Execute.Include(No));

        assert!(Read_write.Include(Read));
        assert!(Read_write.Include(Write));
        assert!(!Read_write.Include(Execute));
        assert!(Read_write.Include(Read_write));
        assert!(!Read_write.Include(Read_execute));
        assert!(!Read_write.Include(Write_execute));
        assert!(!Read_write.Include(Full));
        assert!(Read_write.Include(No));

        assert!(Read_execute.Include(Read));
        assert!(!Read_execute.Include(Write));
        assert!(Read_execute.Include(Execute));
        assert!(!Read_execute.Include(Read_write));
        assert!(Read_execute.Include(Read_execute));
        assert!(!Read_execute.Include(Write_execute));
        assert!(!Read_execute.Include(Full));
        assert!(Read_execute.Include(No));

        assert!(!Write_execute.Include(Read));
        assert!(Write_execute.Include(Write));
        assert!(Write_execute.Include(Execute));
        assert!(!Write_execute.Include(Read_write));
        assert!(!Write_execute.Include(Read_execute));
        assert!(Write_execute.Include(Write_execute));
        assert!(!Write_execute.Include(Full));
        assert!(Write_execute.Include(No));

        assert!(!No.Include(Read));
        assert!(!No.Include(Write));
        assert!(!No.Include(Execute));
        assert!(!No.Include(Read_write));
        assert!(!No.Include(Read_execute));
        assert!(!No.Include(Write_execute));
        assert!(!No.Include(Full));
        assert!(No.Include(No));
    }
}
