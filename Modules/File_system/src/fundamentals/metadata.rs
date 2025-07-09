use users::{Group_identifier_type, User_identifier_type};

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
    r#type: Type_type,
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

    pub fn get_default(
        type_value: Type_type,
        current_time: Time_type,
        user: User_identifier_type,
        group: Group_identifier_type,
    ) -> Option<Self> {
        let permissions = Permissions_type::new_default(type_value);

        Some(Metadata_type {
            inode: None,
            r#type: type_value,
            creation_time: current_time,
            modification_time: current_time,
            access_time: current_time,
            permissions,
            user,
            group,
        })
    }

    pub fn get_inode(&self) -> Option<Inode_type> {
        self.inode
    }

    pub fn get_type(&self) -> Type_type {
        self.r#type
    }

    pub fn get_creation_time(&self) -> Time_type {
        self.creation_time
    }

    pub fn get_modification_time(&self) -> Time_type {
        self.modification_time
    }

    pub fn get_access_time(&self) -> Time_type {
        self.access_time
    }

    pub fn get_permissions(&self) -> Permissions_type {
        self.permissions
    }

    pub fn get_user(&self) -> User_identifier_type {
        self.user
    }

    pub fn get_group(&self) -> Group_identifier_type {
        self.group
    }

    pub fn set_inode(&mut self, inode: Inode_type) {
        self.inode = Some(inode);
    }

    pub fn set_type(&mut self, r#type: Type_type) {
        self.r#type = r#type;
    }

    pub fn set_creation_time(&mut self, time: Time_type) {
        self.creation_time = time;
    }

    pub fn set_modification_time(&mut self, time: Time_type) {
        self.modification_time = time;
    }

    pub fn set_access_time(&mut self, time: Time_type) {
        self.access_time = time;
    }

    pub fn set_permissions(&mut self, permissions: Permissions_type) {
        self.permissions = permissions;
    }

    pub fn set_owner(&mut self, owner: User_identifier_type) {
        self.user = owner;
    }

    pub fn set_group(&mut self, group: Group_identifier_type) {
        self.group = group;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> Metadata_type {
        let current_time = Time_type::new(1640995200);
        let user = User_identifier_type::new(1000);
        let group = Group_identifier_type::new(1000);

        Metadata_type::get_default(Type_type::File, current_time, user, group).unwrap()
    }

    #[test]
    fn test_metadata_creation() {
        let current_time = Time_type::new(1640995200);
        let user = User_identifier_type::new(1000);
        let group = Group_identifier_type::new(1000);

        let metadata = Metadata_type::get_default(Type_type::File, current_time, user, group);
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.get_type(), Type_type::File);
        assert_eq!(metadata.get_creation_time(), current_time);
        assert_eq!(metadata.get_modification_time(), current_time);
        assert_eq!(metadata.get_access_time(), current_time);
        assert_eq!(metadata.get_user(), user);
        assert_eq!(metadata.get_group(), group);
        assert!(metadata.get_inode().is_none());
    }

    #[test]
    fn test_metadata_identifier() {
        assert_eq!(Metadata_type::IDENTIFIER, 0x01);
    }

    #[test]
    fn test_metadata_getters() {
        let metadata = create_test_metadata();

        // Test initial values
        assert!(metadata.get_inode().is_none());
        assert_eq!(metadata.get_type(), Type_type::File);
        assert_eq!(metadata.get_creation_time().as_u64(), 1640995200);
        assert_eq!(metadata.get_modification_time().as_u64(), 1640995200);
        assert_eq!(metadata.get_access_time().as_u64(), 1640995200);
        assert_eq!(metadata.get_user().as_u16(), 1000);
        assert_eq!(metadata.get_group().as_u16(), 1000);
    }

    #[test]
    fn test_metadata_setters() {
        let mut metadata = create_test_metadata();

        // Test setting inode
        let inode = Inode_type::new(42);
        metadata.set_inode(inode);
        assert_eq!(metadata.get_inode(), Some(inode));

        // Test setting type
        metadata.set_type(Type_type::Directory);
        assert_eq!(metadata.get_type(), Type_type::Directory);

        // Test setting times
        let new_time = Time_type::new(1641081600);
        metadata.set_creation_time(new_time);
        metadata.set_modification_time(new_time);
        metadata.set_access_time(new_time);

        assert_eq!(metadata.get_creation_time(), new_time);
        assert_eq!(metadata.get_modification_time(), new_time);
        assert_eq!(metadata.get_access_time(), new_time);

        // Test setting owner and group
        let new_user = User_identifier_type::new(2000);
        let new_group = Group_identifier_type::new(2000);

        metadata.set_owner(new_user);
        metadata.set_group(new_group);

        assert_eq!(metadata.get_user(), new_user);
        assert_eq!(metadata.get_group(), new_group);
    }

    #[test]
    fn test_metadata_permissions() {
        let metadata = create_test_metadata();
        let _permissions = metadata.get_permissions();

        // Test that we can set new permissions
        let mut metadata = metadata;
        let new_permissions = Permissions_type::new_default(Type_type::Directory);
        metadata.set_permissions(new_permissions);

        assert_eq!(metadata.get_permissions(), new_permissions);
    }

    #[test]
    fn test_metadata_clone() {
        let original = create_test_metadata();
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.get_type(), cloned.get_type());
        assert_eq!(original.get_creation_time(), cloned.get_creation_time());
        assert_eq!(original.get_user(), cloned.get_user());
        assert_eq!(original.get_group(), cloned.get_group());
    }

    #[test]
    fn test_metadata_equality() {
        let metadata1 = create_test_metadata();
        let metadata2 = create_test_metadata();

        assert_eq!(metadata1, metadata2);

        // Change one field and verify they're different
        let mut metadata3 = create_test_metadata();
        metadata3.set_type(Type_type::Directory);

        assert_ne!(metadata1, metadata3);
    }

    #[test]
    fn test_metadata_debug() {
        let metadata = create_test_metadata();
        let debug_str = alloc::format!("{metadata:?}");

        assert!(debug_str.contains("Metadata_type"));
        assert!(debug_str.contains("File"));
        assert!(debug_str.contains("1640995200"));
    }

    #[test]
    fn test_metadata_different_types() {
        let current_time = Time_type::new(1640995200);
        let user = User_identifier_type::new(1000);
        let group = Group_identifier_type::new(1000);

        // Test different file types
        let file_metadata =
            Metadata_type::get_default(Type_type::File, current_time, user, group).unwrap();
        let dir_metadata =
            Metadata_type::get_default(Type_type::Directory, current_time, user, group).unwrap();
        let symlink_metadata =
            Metadata_type::get_default(Type_type::Symbolic_link, current_time, user, group)
                .unwrap();

        assert_eq!(file_metadata.get_type(), Type_type::File);
        assert_eq!(dir_metadata.get_type(), Type_type::Directory);
        assert_eq!(symlink_metadata.get_type(), Type_type::Symbolic_link);

        // They should have different permissions based on type
        assert_ne!(
            file_metadata.get_permissions(),
            dir_metadata.get_permissions()
        );
    }

    #[test]
    fn test_metadata_inode_operations() {
        let mut metadata = create_test_metadata();

        // Initially no inode
        assert!(metadata.get_inode().is_none());

        // Set an inode
        let inode1 = Inode_type::new(42);
        metadata.set_inode(inode1);
        assert_eq!(metadata.get_inode(), Some(inode1));

        // Change the inode
        let inode2 = Inode_type::new(84);
        metadata.set_inode(inode2);
        assert_eq!(metadata.get_inode(), Some(inode2));
        assert_ne!(metadata.get_inode(), Some(inode1));
    }

    #[test]
    fn test_metadata_time_updates() {
        let mut metadata = create_test_metadata();

        let initial_time = metadata.get_creation_time();
        let new_time = Time_type::new(initial_time.as_u64() + 3600); // 1 hour later

        // Test that times can be updated independently
        metadata.set_creation_time(new_time);
        assert_eq!(metadata.get_creation_time(), new_time);
        assert_eq!(metadata.get_modification_time(), initial_time); // Should be unchanged
        assert_eq!(metadata.get_access_time(), initial_time); // Should be unchanged

        metadata.set_modification_time(new_time);
        assert_eq!(metadata.get_modification_time(), new_time);
        assert_eq!(metadata.get_access_time(), initial_time); // Should still be unchanged

        metadata.set_access_time(new_time);
        assert_eq!(metadata.get_access_time(), new_time);
    }

    #[test]
    fn test_metadata_user_group_operations() {
        let mut metadata = create_test_metadata();

        let _initial_user = metadata.get_user();
        let initial_group = metadata.get_group();

        let new_user = User_identifier_type::new(5000);
        let new_group = Group_identifier_type::new(5000);

        // Test user change
        metadata.set_owner(new_user);
        assert_eq!(metadata.get_user(), new_user);
        assert_eq!(metadata.get_group(), initial_group); // Group should be unchanged

        // Test group change
        metadata.set_group(new_group);
        assert_eq!(metadata.get_group(), new_group);
        assert_eq!(metadata.get_user(), new_user); // User should remain changed
    }

    #[test]
    fn test_metadata_comprehensive_modification() {
        let mut metadata = create_test_metadata();

        // Modify all fields
        let new_inode = Inode_type::new(999);
        let new_type = Type_type::Socket;
        let new_time = Time_type::new(2000000000);
        let new_user = User_identifier_type::new(9999);
        let new_group = Group_identifier_type::new(9999);
        let new_permissions = Permissions_type::new_default(Type_type::Socket);

        metadata.set_inode(new_inode);
        metadata.set_type(new_type);
        metadata.set_creation_time(new_time);
        metadata.set_modification_time(new_time);
        metadata.set_access_time(new_time);
        metadata.set_owner(new_user);
        metadata.set_group(new_group);
        metadata.set_permissions(new_permissions);

        // Verify all changes
        assert_eq!(metadata.get_inode(), Some(new_inode));
        assert_eq!(metadata.get_type(), new_type);
        assert_eq!(metadata.get_creation_time(), new_time);
        assert_eq!(metadata.get_modification_time(), new_time);
        assert_eq!(metadata.get_access_time(), new_time);
        assert_eq!(metadata.get_user(), new_user);
        assert_eq!(metadata.get_group(), new_group);
        assert_eq!(metadata.get_permissions(), new_permissions);
    }
}
