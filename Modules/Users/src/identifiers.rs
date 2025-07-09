use core::ops::{Add, AddAssign};

pub type User_identifier_inner_type = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct User_identifier_type(User_identifier_inner_type);

impl User_identifier_type {
    pub const ROOT: Self = Self::new(0);

    pub const MINIMUM: Self = Self::new(1);
    pub const MAXIMUM: Self = Self::new(User_identifier_inner_type::MAX);

    pub const fn new(identifier: User_identifier_inner_type) -> Self {
        Self(identifier)
    }

    pub const fn as_u16(self) -> User_identifier_inner_type {
        self.0
    }
}

impl AddAssign<User_identifier_inner_type> for User_identifier_type {
    fn add_assign(&mut self, other: User_identifier_inner_type) {
        self.0 += other;
    }
}

impl Add<User_identifier_inner_type> for User_identifier_type {
    type Output = Self;

    fn add(self, other: User_identifier_inner_type) -> Self {
        Self::new(self.0 + other)
    }
}

impl From<User_identifier_inner_type> for User_identifier_type {
    fn from(value: User_identifier_inner_type) -> Self {
        User_identifier_type::new(value)
    }
}
impl From<User_identifier_type> for User_identifier_inner_type {
    fn from(value: User_identifier_type) -> Self {
        value.as_u16()
    }
}

pub type Group_identifier_inner_type = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Group_identifier_type(Group_identifier_inner_type);

impl Group_identifier_type {
    pub const ROOT: Self = Self::new(0);

    pub const MINIMUM: Self = Self::new(1);
    pub const MAXIMUM: Self = Self::new(Group_identifier_inner_type::MAX);

    pub const fn new(identifier: Group_identifier_inner_type) -> Self {
        Self(identifier)
    }

    pub const fn as_u16(self) -> Group_identifier_inner_type {
        self.0
    }
}

impl From<Group_identifier_inner_type> for Group_identifier_type {
    fn from(value: Group_identifier_inner_type) -> Self {
        Group_identifier_type::new(value)
    }
}
impl From<Group_identifier_type> for Group_identifier_inner_type {
    fn from(value: Group_identifier_type) -> Self {
        value.as_u16()
    }
}

impl AddAssign<Group_identifier_inner_type> for Group_identifier_type {
    fn add_assign(&mut self, other: Group_identifier_inner_type) {
        self.0 += other;
    }
}

impl Add<Group_identifier_inner_type> for Group_identifier_type {
    type Output = Self;

    fn add(self, other: Group_identifier_inner_type) -> Self {
        Self::new(self.0 + other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{collections::BTreeMap, format};

    #[test]
    fn test_user_identifier_constants() {
        assert_eq!(User_identifier_type::ROOT.as_u16(), 0);
        assert_eq!(User_identifier_type::MINIMUM.as_u16(), 1);
        assert_eq!(User_identifier_type::MAXIMUM.as_u16(), u16::MAX);
    }

    #[test]
    fn test_user_identifier_new() {
        let id = User_identifier_type::new(42);
        assert_eq!(id.as_u16(), 42);
    }

    #[test]
    fn test_user_identifier_as_u16() {
        let id = User_identifier_type::new(123);
        assert_eq!(id.as_u16(), 123);
    }

    #[test]
    fn test_user_identifier_add_assign() {
        let mut id = User_identifier_type::new(10);
        id += 5;
        assert_eq!(id.as_u16(), 15);
    }

    #[test]
    fn test_user_identifier_add() {
        let id1 = User_identifier_type::new(10);
        let id2 = id1 + 5;
        assert_eq!(id1.as_u16(), 10); // Original unchanged
        assert_eq!(id2.as_u16(), 15);
    }

    #[test]
    fn test_user_identifier_from_u16() {
        let id: User_identifier_type = 42u16.into();
        assert_eq!(id.as_u16(), 42);
    }

    #[test]
    fn test_user_identifier_to_u16() {
        let id = User_identifier_type::new(42);
        let value: u16 = id.into();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_user_identifier_clone_copy() {
        let id1 = User_identifier_type::new(42);
        let id2 = id1; // Copy
        let id3 = id1; // Copy (Clone not needed for Copy types)

        assert_eq!(id1.as_u16(), 42);
        assert_eq!(id2.as_u16(), 42);
        assert_eq!(id3.as_u16(), 42);
    }

    #[test]
    fn test_user_identifier_equality() {
        let id1 = User_identifier_type::new(42);
        let id2 = User_identifier_type::new(42);
        let id3 = User_identifier_type::new(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_user_identifier_ordering() {
        let id1 = User_identifier_type::new(10);
        let id2 = User_identifier_type::new(20);

        assert!(id1 < id2);
        assert!(id2 > id1);
        assert!(id1 <= id2);
        assert!(id2 >= id1);
        assert!(id1 <= id1);
        assert!(id1 >= id1);
    }

    #[test]
    fn test_group_identifier_constants() {
        assert_eq!(Group_identifier_type::ROOT.as_u16(), 0);
        assert_eq!(Group_identifier_type::MINIMUM.as_u16(), 1);
        assert_eq!(Group_identifier_type::MAXIMUM.as_u16(), u16::MAX);
    }

    #[test]
    fn test_group_identifier_new() {
        let id = Group_identifier_type::new(42);
        assert_eq!(id.as_u16(), 42);
    }

    #[test]
    fn test_group_identifier_as_u16() {
        let id = Group_identifier_type::new(123);
        assert_eq!(id.as_u16(), 123);
    }

    #[test]
    fn test_group_identifier_add_assign() {
        let mut id = Group_identifier_type::new(10);
        id += 5;
        assert_eq!(id.as_u16(), 15);
    }

    #[test]
    fn test_group_identifier_add() {
        let id1 = Group_identifier_type::new(10);
        let id2 = id1 + 5;
        assert_eq!(id1.as_u16(), 10); // Original unchanged
        assert_eq!(id2.as_u16(), 15);
    }

    #[test]
    fn test_group_identifier_from_u16() {
        let id: Group_identifier_type = 42u16.into();
        assert_eq!(id.as_u16(), 42);
    }

    #[test]
    fn test_group_identifier_to_u16() {
        let id = Group_identifier_type::new(42);
        let value: u16 = id.into();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_group_identifier_clone_copy() {
        let id1 = Group_identifier_type::new(42);
        let id2 = id1; // Copy
        let id3 = id1; // Copy (Clone not needed for Copy types)

        assert_eq!(id1.as_u16(), 42);
        assert_eq!(id2.as_u16(), 42);
        assert_eq!(id3.as_u16(), 42);
    }

    #[test]
    fn test_group_identifier_equality() {
        let id1 = Group_identifier_type::new(42);
        let id2 = Group_identifier_type::new(42);
        let id3 = Group_identifier_type::new(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_group_identifier_ordering() {
        let id1 = Group_identifier_type::new(10);
        let id2 = Group_identifier_type::new(20);

        assert!(id1 < id2);
        assert!(id2 > id1);
        assert!(id1 <= id2);
        assert!(id2 >= id1);
        assert!(id1 <= id1);
        assert!(id1 >= id1);
    }

    #[test]
    fn test_edge_cases_user_identifier() {
        // Test minimum value
        let min_id = User_identifier_type::new(0);
        assert_eq!(min_id.as_u16(), 0);

        // Test maximum value
        let max_id = User_identifier_type::new(u16::MAX);
        assert_eq!(max_id.as_u16(), u16::MAX);

        // Test overflow behavior (this will wrap around in debug mode)
        let near_max = User_identifier_type::new(u16::MAX - 1);
        let result = near_max + 1;
        assert_eq!(result.as_u16(), u16::MAX);
    }

    #[test]
    fn test_edge_cases_group_identifier() {
        // Test minimum value
        let min_id = Group_identifier_type::new(0);
        assert_eq!(min_id.as_u16(), 0);

        // Test maximum value
        let max_id = Group_identifier_type::new(u16::MAX);
        assert_eq!(max_id.as_u16(), u16::MAX);

        // Test overflow behavior (this will wrap around in debug mode)
        let near_max = Group_identifier_type::new(u16::MAX - 1);
        let result = near_max + 1;
        assert_eq!(result.as_u16(), u16::MAX);
    }

    #[test]
    fn test_hash_consistency() {
        let mut user_map = BTreeMap::new();
        let mut group_map = BTreeMap::new();

        let user_id = User_identifier_type::new(42);
        let group_id = Group_identifier_type::new(42);

        user_map.insert(user_id, "test_user");
        group_map.insert(group_id, "test_group");

        assert_eq!(user_map.get(&user_id), Some(&"test_user"));
        assert_eq!(group_map.get(&group_id), Some(&"test_group"));
    }

    #[test]
    fn test_debug_format() {
        let user_id = User_identifier_type::new(42);
        let group_id = Group_identifier_type::new(42);

        let user_debug = format!("{user_id:?}");
        let group_debug = format!("{group_id:?}");

        assert!(user_debug.contains("42"));
        assert!(group_debug.contains("42"));
    }
}
