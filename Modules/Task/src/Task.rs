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
    pub const fn New(Identifier: Task_identifier_inner_type) -> Self {
        Self(Identifier)
    }

    pub const fn Into_inner(self) -> Task_identifier_inner_type {
        self.0
    }
}

impl From<Task_identifier_inner_type> for Task_identifier_type {
    fn from(Value: Task_identifier_inner_type) -> Self {
        Self(Value)
    }
}

impl From<Task_identifier_type> for Task_identifier_inner_type {
    fn from(Value: Task_identifier_type) -> Self {
        Value.0
    }
}

impl PartialEq<Task_identifier_inner_type> for Task_identifier_type {
    fn eq(&self, other: &Task_identifier_inner_type) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod Tests {
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
        let id = Task_identifier_type::New(42);
        assert_eq!(id.Into_inner(), 42);
    }

    #[test]
    fn test_task_identifier_into_inner() {
        let inner_value = 123;
        let id = Task_identifier_type::New(inner_value);
        assert_eq!(id.Into_inner(), inner_value);
    }

    #[test]
    fn test_task_identifier_from_inner_type() {
        let inner_value = 456;
        let id: Task_identifier_type = inner_value.into();
        assert_eq!(id.Into_inner(), inner_value);
    }

    #[test]
    fn test_task_identifier_into_inner_type() {
        let id = Task_identifier_type::New(789);
        let inner_value: Task_identifier_inner_type = id.into();
        assert_eq!(inner_value, 789);
    }

    #[test]
    fn test_task_identifier_clone_copy() {
        let id1 = Task_identifier_type::New(42);
        let id2 = id1; // Copy
        let id3 = id1; // Copy (Clone not needed for Copy types)

        assert_eq!(id1.Into_inner(), 42);
        assert_eq!(id2.Into_inner(), 42);
        assert_eq!(id3.Into_inner(), 42);
    }

    #[test]
    fn test_task_identifier_equality() {
        let id1 = Task_identifier_type::New(42);
        let id2 = Task_identifier_type::New(42);
        let id3 = Task_identifier_type::New(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_task_identifier_equality_with_inner_type() {
        let id = Task_identifier_type::New(42);
        let inner_value = 42;
        let different_value = 43;

        assert_eq!(id, inner_value);
        assert_ne!(id, different_value);
    }

    #[test]
    fn test_task_identifier_ordering() {
        let id1 = Task_identifier_type::New(10);
        let id2 = Task_identifier_type::New(20);

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
        let id = Task_identifier_type::New(42);

        map.insert(id, "test_task");
        assert_eq!(map.get(&id), Some(&"test_task"));
    }

    #[test]
    fn test_task_identifier_debug_format() {
        let id = Task_identifier_type::New(42);
        let debug_str = format!("{id:?}");
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_edge_cases() {
        // Test minimum value
        let min_id = Task_identifier_type::New(Task_identifier_type::MINIMUM);
        assert_eq!(min_id.Into_inner(), Task_identifier_type::MINIMUM);

        // Test maximum value
        let max_id = Task_identifier_type::New(Task_identifier_type::MAXIMUM);
        assert_eq!(max_id.Into_inner(), Task_identifier_type::MAXIMUM);
    }

    #[test]
    fn test_const_constructor() {
        // Test that the constructor can be used in const context
        const ID: Task_identifier_type = Task_identifier_type::New(100);
        assert_eq!(ID.Into_inner(), 100);
    }

    #[test]
    fn test_const_into_inner() {
        // Test that Into_inner can be used in const context
        const ID: Task_identifier_type = Task_identifier_type::New(200);
        const INNER: Task_identifier_inner_type = ID.Into_inner();
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
        let zero_id = Task_identifier_type::New(0);
        assert_eq!(zero_id.Into_inner(), 0);
        assert_eq!(zero_id, 0);
    }

    #[test]
    fn test_bidirectional_conversion() {
        let original_value = 12345;
        let id = Task_identifier_type::New(original_value);
        let converted_back: Task_identifier_inner_type = id.into();
        let id_again: Task_identifier_type = converted_back.into();

        assert_eq!(original_value, converted_back);
        assert_eq!(id, id_again);
    }
}
