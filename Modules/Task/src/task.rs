#[cfg(target_pointer_width = "32")]
pub type Task_identifier_inner_type = u16;
#[cfg(target_pointer_width = "64")]
pub type Task_identifier_inner_type = u32;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Task_identifier_type(Task_identifier_inner_type);

impl Task_identifier_type {
    pub const MAXIMUM: Task_identifier_inner_type = Task_identifier_inner_type::MAX;
    pub const MINIMUM: Task_identifier_inner_type = Task_identifier_inner_type::MIN;
}

impl Task_identifier_type {
    pub const fn new(identifier: Task_identifier_inner_type) -> Self {
        Self(identifier)
    }

    pub const fn into_inner(self) -> Task_identifier_inner_type {
        self.0
    }
}

impl From<Task_identifier_inner_type> for Task_identifier_type {
    fn from(value: Task_identifier_inner_type) -> Self {
        Self(value)
    }
}

impl From<Task_identifier_type> for Task_identifier_inner_type {
    fn from(value: Task_identifier_type) -> Self {
        value.0
    }
}

impl PartialEq<Task_identifier_inner_type> for Task_identifier_type {
    fn eq(&self, other: &Task_identifier_inner_type) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::format;

    #[test]
    fn test_task_identifier_constants() {
        assert_eq!(
            Task_identifier_type::MAXIMUM,
            Task_identifier_inner_type::MAX
        );
        assert_eq!(
            Task_identifier_type::MINIMUM,
            Task_identifier_inner_type::MIN
        );
    }

    #[test]
    fn test_task_identifier_new() {
        let id = Task_identifier_type::new(42);
        assert_eq!(id.into_inner(), 42);
    }

    #[test]
    fn test_task_identifier_into_inner() {
        let inner_value = 123;
        let id = Task_identifier_type::new(inner_value);
        assert_eq!(id.into_inner(), inner_value);
    }

    #[test]
    fn test_task_identifier_from_inner_type() {
        let inner_value = 456;
        let id: Task_identifier_type = inner_value.into();
        assert_eq!(id.into_inner(), inner_value);
    }

    #[test]
    fn test_task_identifier_into_inner_type() {
        let id = Task_identifier_type::new(789);
        let inner_value: Task_identifier_inner_type = id.into();
        assert_eq!(inner_value, 789);
    }

    #[test]
    fn test_task_identifier_clone_copy() {
        let id1 = Task_identifier_type::new(42);
        let id2 = id1; // Copy
        let id3 = id1; // Copy (Clone not needed for Copy types)

        assert_eq!(id1.into_inner(), 42);
        assert_eq!(id2.into_inner(), 42);
        assert_eq!(id3.into_inner(), 42);
    }

    #[test]
    fn test_task_identifier_equality() {
        let id1 = Task_identifier_type::new(42);
        let id2 = Task_identifier_type::new(42);
        let id3 = Task_identifier_type::new(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_task_identifier_equality_with_inner_type() {
        let id = Task_identifier_type::new(42);
        let inner_value = 42;
        let different_value = 43;

        assert_eq!(id, inner_value);
        assert_ne!(id, different_value);
    }

    #[test]
    fn test_task_identifier_ordering() {
        let id1 = Task_identifier_type::new(10);
        let id2 = Task_identifier_type::new(20);

        assert!(id1 < id2);
        assert!(id2 > id1);
        assert!(id1 <= id2);
        assert!(id2 >= id1);
        assert!(id1 <= id1);
        assert!(id1 >= id1);
    }

    #[test]
    fn test_task_identifier_hash_consistency() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let id = Task_identifier_type::new(42);

        map.insert(id, "test_task");
        assert_eq!(map.get(&id), Some(&"test_task"));
    }

    #[test]
    fn test_task_identifier_debug_format() {
        let id = Task_identifier_type::new(42);
        let debug_str = format!("{id:?}");
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_edge_cases() {
        // Test minimum value
        let min_id = Task_identifier_type::new(Task_identifier_type::MINIMUM);
        assert_eq!(min_id.into_inner(), Task_identifier_type::MINIMUM);

        // Test maximum value
        let max_id = Task_identifier_type::new(Task_identifier_type::MAXIMUM);
        assert_eq!(max_id.into_inner(), Task_identifier_type::MAXIMUM);
    }

    #[test]
    fn test_const_constructor() {
        // Test that the constructor can be used in const context
        const ID: Task_identifier_type = Task_identifier_type::new(100);
        assert_eq!(ID.into_inner(), 100);
    }

    #[test]
    fn test_const_into_inner() {
        // Test that Into_inner can be used in const context
        const ID: Task_identifier_type = Task_identifier_type::new(200);
        const INNER: Task_identifier_inner_type = ID.into_inner();
        assert_eq!(INNER, 200);
    }

    #[test]
    fn test_repr_transparent() {
        // Test that the size is the same as the inner type (due to #[repr(transparent)])
        assert_eq!(
            std::mem::size_of::<Task_identifier_type>(),
            std::mem::size_of::<Task_identifier_inner_type>()
        );
    }

    #[test]
    fn test_different_architectures() {
        // This test verifies the conditional compilation works correctly
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<Task_identifier_inner_type>(), 2); // u16
        }

        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<Task_identifier_inner_type>(), 4); // u32
        }
    }

    #[test]
    fn test_zero_value() {
        let zero_id = Task_identifier_type::new(0);
        assert_eq!(zero_id.into_inner(), 0);
        assert_eq!(zero_id, 0);
    }

    #[test]
    fn test_bidirectional_conversion() {
        let original_value = 12345;
        let id = Task_identifier_type::new(original_value);
        let converted_back: Task_identifier_inner_type = id.into();
        let id_again: Task_identifier_type = converted_back.into();

        assert_eq!(original_value, converted_back);
        assert_eq!(id, id_again);
    }
}
