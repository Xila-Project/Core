#[cfg(target_pointer_width = "32")]
pub type Task_identifier_inner_type = u16;
#[cfg(target_pointer_width = "64")]
pub type Task_identifier_inner_type = u32;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Task_identifier_type(Task_identifier_inner_type);

impl Task_identifier_type {
    pub const Maximum: Task_identifier_inner_type = Task_identifier_inner_type::MAX;
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
