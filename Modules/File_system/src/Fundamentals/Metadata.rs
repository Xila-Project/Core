use Users::{Group_identifier_type, User_identifier_type};

use crate::{Permissions_type, Time_type, Type_type};

use super::Inode_type;

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
pub struct Metadata_type {
    /// The file inode.
    inode: Option<Inode_type>,
    /// The file type.
    Type: Type_type,
    /// The file creation time.
    creation_time: Time_type,
    /// The file modification time.
    modification_time: Time_type,
    /// The file access time.
    access_time: Time_type,
    /// The file permissions.
    permissions: Permissions_type,
    /// The file owner.
    user: User_identifier_type,
    /// The file group.
    group: Group_identifier_type,
}

impl Metadata_type {
    pub const IDENTIFIER: u8 = 0x01;

    pub fn Get_default(
        type_value: Type_type,
        current_time: Time_type,
        user: User_identifier_type,
        group: Group_identifier_type,
    ) -> Option<Self> {
        let permissions = Permissions_type::New_default(type_value);

        Some(Metadata_type {
            inode: None,
            Type: type_value,
            creation_time: current_time,
            modification_time: current_time,
            access_time: current_time,
            permissions,
            user,
            group,
        })
    }

    pub fn Get_inode(&self) -> Option<Inode_type> {
        self.inode
    }

    pub fn Get_type(&self) -> Type_type {
        self.Type
    }

    pub fn Get_creation_time(&self) -> Time_type {
        self.creation_time
    }

    pub fn Get_modification_time(&self) -> Time_type {
        self.modification_time
    }

    pub fn Get_access_time(&self) -> Time_type {
        self.access_time
    }

    pub fn Get_permissions(&self) -> Permissions_type {
        self.permissions
    }

    pub fn Get_user(&self) -> User_identifier_type {
        self.user
    }

    pub fn Get_group(&self) -> Group_identifier_type {
        self.group
    }

    pub fn Set_inode(&mut self, Inode: Inode_type) {
        self.inode = Some(Inode);
    }

    pub fn Set_type(&mut self, Type: Type_type) {
        self.Type = Type;
    }

    pub fn Set_creation_time(&mut self, Time: Time_type) {
        self.creation_time = Time;
    }

    pub fn Set_modification_time(&mut self, Time: Time_type) {
        self.modification_time = Time;
    }

    pub fn Set_access_time(&mut self, Time: Time_type) {
        self.access_time = Time;
    }

    pub fn Set_permissions(&mut self, Permissions: Permissions_type) {
        self.permissions = Permissions;
    }

    pub fn Set_owner(&mut self, Owner: User_identifier_type) {
        self.user = Owner;
    }

    pub fn Set_group(&mut self, Group: Group_identifier_type) {
        self.group = Group;
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    fn Create_test_metadata() -> Metadata_type {
        let current_time = Time_type::New(1640995200);
        let user = User_identifier_type::New(1000);
        let group = Group_identifier_type::New(1000);

        Metadata_type::Get_default(Type_type::File, current_time, user, group).unwrap()
    }

    #[test]
    fn Test_metadata_creation() {
        let current_time = Time_type::New(1640995200);
        let user = User_identifier_type::New(1000);
        let group = Group_identifier_type::New(1000);

        let metadata = Metadata_type::Get_default(Type_type::File, current_time, user, group);
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.Get_type(), Type_type::File);
        assert_eq!(metadata.Get_creation_time(), current_time);
        assert_eq!(metadata.Get_modification_time(), current_time);
        assert_eq!(metadata.Get_access_time(), current_time);
        assert_eq!(metadata.Get_user(), user);
        assert_eq!(metadata.Get_group(), group);
        assert!(metadata.Get_inode().is_none());
    }

    #[test]
    fn Test_metadata_identifier() {
        assert_eq!(Metadata_type::IDENTIFIER, 0x01);
    }

    #[test]
    fn Test_metadata_getters() {
        let metadata = Create_test_metadata();

        // Test initial values
        assert!(metadata.Get_inode().is_none());
        assert_eq!(metadata.Get_type(), Type_type::File);
        assert_eq!(metadata.Get_creation_time().As_u64(), 1640995200);
        assert_eq!(metadata.Get_modification_time().As_u64(), 1640995200);
        assert_eq!(metadata.Get_access_time().As_u64(), 1640995200);
        assert_eq!(metadata.Get_user().As_u16(), 1000);
        assert_eq!(metadata.Get_group().As_u16(), 1000);
    }

    #[test]
    fn Test_metadata_setters() {
        let mut metadata = Create_test_metadata();

        // Test setting inode
        let inode = Inode_type::New(42);
        metadata.Set_inode(inode);
        assert_eq!(metadata.Get_inode(), Some(inode));

        // Test setting type
        metadata.Set_type(Type_type::Directory);
        assert_eq!(metadata.Get_type(), Type_type::Directory);

        // Test setting times
        let new_time = Time_type::New(1641081600);
        metadata.Set_creation_time(new_time);
        metadata.Set_modification_time(new_time);
        metadata.Set_access_time(new_time);

        assert_eq!(metadata.Get_creation_time(), new_time);
        assert_eq!(metadata.Get_modification_time(), new_time);
        assert_eq!(metadata.Get_access_time(), new_time);

        // Test setting owner and group
        let new_user = User_identifier_type::New(2000);
        let new_group = Group_identifier_type::New(2000);

        metadata.Set_owner(new_user);
        metadata.Set_group(new_group);

        assert_eq!(metadata.Get_user(), new_user);
        assert_eq!(metadata.Get_group(), new_group);
    }

    #[test]
    fn Test_metadata_permissions() {
        let metadata = Create_test_metadata();
        let _permissions = metadata.Get_permissions();

        // Test that we can set new permissions
        let mut metadata = metadata;
        let new_permissions = Permissions_type::New_default(Type_type::Directory);
        metadata.Set_permissions(new_permissions);

        assert_eq!(metadata.Get_permissions(), new_permissions);
    }

    #[test]
    fn Test_metadata_clone() {
        let original = Create_test_metadata();
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.Get_type(), cloned.Get_type());
        assert_eq!(original.Get_creation_time(), cloned.Get_creation_time());
        assert_eq!(original.Get_user(), cloned.Get_user());
        assert_eq!(original.Get_group(), cloned.Get_group());
    }

    #[test]
    fn Test_metadata_equality() {
        let metadata1 = Create_test_metadata();
        let metadata2 = Create_test_metadata();

        assert_eq!(metadata1, metadata2);

        // Change one field and verify they're different
        let mut metadata3 = Create_test_metadata();
        metadata3.Set_type(Type_type::Directory);

        assert_ne!(metadata1, metadata3);
    }

    #[test]
    fn Test_metadata_debug() {
        let metadata = Create_test_metadata();
        let debug_str = alloc::format!("{metadata:?}");

        assert!(debug_str.contains("Metadata_type"));
        assert!(debug_str.contains("File"));
        assert!(debug_str.contains("1640995200"));
    }

    #[test]
    fn Test_metadata_different_types() {
        let current_time = Time_type::New(1640995200);
        let user = User_identifier_type::New(1000);
        let group = Group_identifier_type::New(1000);

        // Test different file types
        let file_metadata =
            Metadata_type::Get_default(Type_type::File, current_time, user, group).unwrap();
        let dir_metadata =
            Metadata_type::Get_default(Type_type::Directory, current_time, user, group).unwrap();
        let symlink_metadata =
            Metadata_type::Get_default(Type_type::Symbolic_link, current_time, user, group)
                .unwrap();

        assert_eq!(file_metadata.Get_type(), Type_type::File);
        assert_eq!(dir_metadata.Get_type(), Type_type::Directory);
        assert_eq!(symlink_metadata.Get_type(), Type_type::Symbolic_link);

        // They should have different permissions based on type
        assert_ne!(
            file_metadata.Get_permissions(),
            dir_metadata.Get_permissions()
        );
    }

    #[test]
    fn Test_metadata_inode_operations() {
        let mut metadata = Create_test_metadata();

        // Initially no inode
        assert!(metadata.Get_inode().is_none());

        // Set an inode
        let inode1 = Inode_type::New(42);
        metadata.Set_inode(inode1);
        assert_eq!(metadata.Get_inode(), Some(inode1));

        // Change the inode
        let inode2 = Inode_type::New(84);
        metadata.Set_inode(inode2);
        assert_eq!(metadata.Get_inode(), Some(inode2));
        assert_ne!(metadata.Get_inode(), Some(inode1));
    }

    #[test]
    fn Test_metadata_time_updates() {
        let mut metadata = Create_test_metadata();

        let initial_time = metadata.Get_creation_time();
        let new_time = Time_type::New(initial_time.As_u64() + 3600); // 1 hour later

        // Test that times can be updated independently
        metadata.Set_creation_time(new_time);
        assert_eq!(metadata.Get_creation_time(), new_time);
        assert_eq!(metadata.Get_modification_time(), initial_time); // Should be unchanged
        assert_eq!(metadata.Get_access_time(), initial_time); // Should be unchanged

        metadata.Set_modification_time(new_time);
        assert_eq!(metadata.Get_modification_time(), new_time);
        assert_eq!(metadata.Get_access_time(), initial_time); // Should still be unchanged

        metadata.Set_access_time(new_time);
        assert_eq!(metadata.Get_access_time(), new_time);
    }

    #[test]
    fn Test_metadata_user_group_operations() {
        let mut metadata = Create_test_metadata();

        let _initial_user = metadata.Get_user();
        let initial_group = metadata.Get_group();

        let new_user = User_identifier_type::New(5000);
        let new_group = Group_identifier_type::New(5000);

        // Test user change
        metadata.Set_owner(new_user);
        assert_eq!(metadata.Get_user(), new_user);
        assert_eq!(metadata.Get_group(), initial_group); // Group should be unchanged

        // Test group change
        metadata.Set_group(new_group);
        assert_eq!(metadata.Get_group(), new_group);
        assert_eq!(metadata.Get_user(), new_user); // User should remain changed
    }

    #[test]
    fn Test_metadata_comprehensive_modification() {
        let mut metadata = Create_test_metadata();

        // Modify all fields
        let new_inode = Inode_type::New(999);
        let new_type = Type_type::Socket;
        let new_time = Time_type::New(2000000000);
        let new_user = User_identifier_type::New(9999);
        let new_group = Group_identifier_type::New(9999);
        let new_permissions = Permissions_type::New_default(Type_type::Socket);

        metadata.Set_inode(new_inode);
        metadata.Set_type(new_type);
        metadata.Set_creation_time(new_time);
        metadata.Set_modification_time(new_time);
        metadata.Set_access_time(new_time);
        metadata.Set_owner(new_user);
        metadata.Set_group(new_group);
        metadata.Set_permissions(new_permissions);

        // Verify all changes
        assert_eq!(metadata.Get_inode(), Some(new_inode));
        assert_eq!(metadata.Get_type(), new_type);
        assert_eq!(metadata.Get_creation_time(), new_time);
        assert_eq!(metadata.Get_modification_time(), new_time);
        assert_eq!(metadata.Get_access_time(), new_time);
        assert_eq!(metadata.Get_user(), new_user);
        assert_eq!(metadata.Get_group(), new_group);
        assert_eq!(metadata.Get_permissions(), new_permissions);
    }
}
