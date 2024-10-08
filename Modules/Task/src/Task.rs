use Users::User_identifier_type;

use crate::{Environment_variable_type, Get_instance, Join_handle_type, Result_type};

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

/// A wrapper for individual tasks that are managed by [Manager_type].
pub struct Task_type {
    /// The identifier of the task.
    Identifier: Task_identifier_type,
}

impl Task_type {
    /// Internal method to create a new task.
    pub(crate) fn New(Identifier: Task_identifier_type) -> Self {
        Self { Identifier }
    }

    /// Create a new child task.
    pub fn New_child_task<T, F>(
        &self,
        Name: &str,
        Owner: Option<User_identifier_type>,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result_type<(Task_type, Join_handle_type<T>)>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let (Task_identifier, Join_handle) =
            Get_instance().New_task(self.Identifier, Owner, Name, Stack_size, Function)?;

        Ok((Task_type::New(Task_identifier), Join_handle))
    }

    pub fn Get_current_task() -> Result_type<Self> {
        let Identifier = Get_instance().Get_current_task_identifier()?;

        Ok(Self { Identifier })
    }

    pub fn Get_name(&self) -> Result_type<String> {
        Get_instance().Get_task_name(self.Identifier)
    }

    pub fn Get_identifier(&self) -> Task_identifier_type {
        self.Identifier
    }

    pub fn Get_owner(&self) -> Result_type<User_identifier_type> {
        Get_instance().Get_owner(self.Identifier)
    }

    pub fn Get_environment_variable(&self, Name: &str) -> Result_type<Environment_variable_type> {
        Get_instance().Get_environment_variable(self.Identifier, Name)
    }

    pub fn Set_environment_variable(&self, Name: &str, Value: &str) -> Result_type<()> {
        Get_instance().Set_environment_variable(self.Identifier, Name, Value)
    }

    pub fn Remove_environment_variable(&self, Name: &str) -> Result_type<()> {
        Get_instance().Remove_environment_variable(self.Identifier, Name)
    }
}
