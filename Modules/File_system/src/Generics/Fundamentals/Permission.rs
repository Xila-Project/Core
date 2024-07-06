#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Permissions_type(u16);

impl Permissions_type {
    /// Creates a new permission.
    pub fn New(User: &Permission_type, Group: &Permission_type, Others: &Permission_type) -> Self {
        Self(0).Set_user(User).Set_group(Group).Set_others(Others)
    }

    /// Creates a new permission with full access (read, write, execute) for all (user, group, others).
    pub fn New_all_full() -> Self {
        Self::New(
            &Permission_type::New_full(),
            &Permission_type::New_full(),
            &Permission_type::New_full(),
        )
    }

    /// Creates a new permission with full access (read, write, execute) for user. No access for group and others.
    pub fn New_user_full() -> Self {
        Self::New(
            &Permission_type::New_full(),
            &Permission_type::New_none(),
            &Permission_type::New_none(),
        )
    }

    /// Creates a new permission with read write access for user. No access for group and others.
    pub fn New_user_read_write() -> Self {
        Self::New(
            &Permission_type::New_read_write(),
            &Permission_type::New_none(),
            &Permission_type::New_none(),
        )
    }

    /// Creates a new permission with read access for user. No access for group and others.
    pub fn New_standard_directory() -> Self {
        Self::New(
            &Permission_type::New_full(),
            &Permission_type::New_write(),
            &Permission_type::New_execute(),
        )
    }

    /// Creates a new permission with read access for user. No access for group and others.
    pub fn New_standard_file() -> Self {
        Self::New(
            &Permission_type::New_full(),
            &Permission_type::New_read(),
            &Permission_type::New_none(),
        )
    }

    /// Sets the permission for the user.
    pub fn Set_user(mut self, User: &Permission_type) -> Self {
        self.0 = (self.0 & 0o077) | (User.To_unix() as u16) << 6;
        self
    }

    /// Sets the permission for the group.
    pub fn Set_group(mut self, Group: &Permission_type) -> Self {
        self.0 = (self.0 & 0o707) | (Group.To_unix() as u16) << 3;
        self
    }

    /// Sets the permission for others.
    pub fn Set_others(mut self, Others: &Permission_type) -> Self {
        self.0 = (self.0 & 0o770) | Others.To_unix() as u16;
        self
    }

    /// Gets the permission for the user.
    pub fn Get_user(&self) -> Permission_type {
        Permission_type::From_unix((self.0 >> 6) as u8).unwrap()
    }

    /// Gets the permission for the group.
    pub fn Get_group(&self) -> Permission_type {
        Permission_type::From_unix(((self.0 >> 3) & 0b111) as u8).unwrap()
    }

    /// Gets the permission for others.
    pub fn Get_others(&self) -> Permission_type {
        Permission_type::From_unix((self.0 & 0b111) as u8).unwrap()
    }

    /// Converts the permission to a Unix permission.
    pub fn From_unix(Unix: u16) -> Option<Self> {
        if Unix > 0o777 {
            return None;
        }

        Some(Self(Unix))
    }

    /// Converts the permission to a Unix permission.
    pub fn To_unix(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Permission_type(u8);

impl Permission_type {
    /// Creates a new permission.
    pub fn New(Read: bool, Write: bool, Execute: bool) -> Self {
        Self((Read as u8) << 2 | (Write as u8) << 1 | Execute as u8)
    }

    /// Creates a new permission with read access (equivalent to Unix permission 4).
    pub fn New_read() -> Self {
        Self::New(true, false, false)
    }

    /// Creates a new permission with write access (equivalent to Unix permission 2).
    pub fn New_write() -> Self {
        Self::New(false, true, false)
    }

    /// Creates a new permission with execute access (equivalent to Unix permission 1).
    pub fn New_execute() -> Self {
        Self::New(false, false, true)
    }

    pub fn New_write_execute() -> Self {
        Self::New(false, true, true)
    }

    pub fn New_read_execute() -> Self {
        Self::New(true, false, true)
    }

    /// Creates a new permission with full access (equivalent to Unix permission 7).
    pub fn New_full() -> Self {
        Self::New(true, true, true)
    }

    /// Creates a new permission with no access (equivalent to Unix permission 0).
    pub fn New_read_write() -> Self {
        Self::New(true, true, false)
    }

    /// Creates a new permission with no access (equivalent to Unix permission 0).
    pub fn New_none() -> Self {
        Self::New(false, false, false)
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
    pub fn Get_read(&self) -> bool {
        self.0 & 0b100 != 0
    }

    /// Gets the write permission.
    pub fn Get_write(&self) -> bool {
        self.0 & 0b010 != 0
    }

    /// Gets the execute permission.
    pub fn Get_execute(&self) -> bool {
        self.0 & 0b001 != 0
    }

    /// Converts the permission to a Unix permission.
    pub fn To_unix(&self) -> u8 {
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
        let permissions = Permissions_type::New(&user, &group, &others);
        assert_eq!(permissions.0, 0b100_010_001);
    }

    #[test]
    fn Test_new_permission() {
        let permissions = Permissions_type::New_all_full();
        assert_eq!(permissions.0, 0b111_111_111);

        let permissions = Permissions_type::New_user_full();
        assert_eq!(permissions.0, 0b111_000_000);

        let permissions = Permissions_type::New_user_read_write();
        assert_eq!(permissions.0, 0b110_000_000);
    }

    #[test]
    fn Test_permission_type_to_unix() {
        let read = Permission_type::New_read();
        assert_eq!(read.To_unix(), 4);
        let write = Permission_type::New_write();
        assert_eq!(write.To_unix(), 2);
        let execute = Permission_type::New_execute();
        assert_eq!(execute.To_unix(), 1);
        let full = Permission_type::New_full();
        assert_eq!(full.To_unix(), 7);
        let none = Permission_type::New_none();
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
        let Permissions = Permissions_type::From_unix(0b101_101_101).unwrap();
        assert_eq!(Permissions.Get_user().To_unix(), 5);
        assert_eq!(Permissions.Get_group().To_unix(), 5);
        assert_eq!(Permissions.Get_others().To_unix(), 5);
    }

    #[test]
    fn Test_permissions_type_to_unix() {
        let User = Permission_type::New(true, false, true); // Read and execute
        let Group = Permission_type::New(true, true, false); // Read and write
        let Others = Permission_type::New(false, true, true); // Write and execute
        let Permissions = Permissions_type::New(&User, &Group, &Others);
        assert_eq!(Permissions.To_unix(), 0b101_110_011);
    }

    #[test]
    fn Test_permission_type_include() {
        let Read = Permission_type::New_read();
        let Write = Permission_type::New_write();
        let Read_write = Permission_type::New_read_write();
        let Read_execute = Permission_type::New_read_execute();
        let Write_execute = Permission_type::New_write_execute();
        let Execute = Permission_type::New_execute();
        let Full = Permission_type::New_full();
        let No = Permission_type::New_none();

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
