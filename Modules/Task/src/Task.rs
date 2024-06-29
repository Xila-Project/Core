use std::borrow::Cow;

use Users::User_identifier_type;

use crate::{Join_handle_type, Manager_type, Result, Thread_wrapper_type};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Task_identifier_type(u32);

impl Task_identifier_type {
    pub const Maximum: u32 = u32::MAX;
}

impl From<u32> for Task_identifier_type {
    fn from(Value: u32) -> Self {
        Self(Value)
    }
}

impl From<Task_identifier_type> for u32 {
    fn from(Value: Task_identifier_type) -> Self {
        Value.0
    }
}

/// A wrapper for individual tasks that are managed by [Manager_type].
pub struct Task_type {
    /// The identifier of the task.
    Identifier: Task_identifier_type,
    /// A reference to the [Manager_type] that manages the task.
    Manager: Manager_type,
}

impl Task_type {
    /// Internal method to create a new task.
    pub(crate) fn New(Identifier: Task_identifier_type, Manager: Manager_type) -> Self {
        Self {
            Identifier,
            Manager,
        }
    }

    /// Create a new child task.
    pub fn New_child_task<T, F>(
        &self,
        Name: &str,
        Owner: Option<User_identifier_type>,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result<(Task_type, Join_handle_type<T>)>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let (Task_identifier, Join_handle) =
            self.Manager
                .New_task(Some(self.Identifier), Owner, Name, Stack_size, Function)?;

        Ok((
            Task_type::New(Task_identifier, self.Manager.clone()),
            Join_handle,
        ))
    }

    pub fn Get_name(&self) -> Result<String> {
        self.Manager.Get_task_name(self.Identifier)
    }

    pub fn Get_identifier(&self) -> Task_identifier_type {
        self.Identifier
    }

    pub fn Get_owner(&self) -> Result<User_identifier_type> {
        self.Manager.Get_owner(self.Identifier)
    }

    pub fn Sleep(Duration: std::time::Duration) {
        Thread_wrapper_type::Sleep(Duration)
    }

    pub fn Get_environment_variable(&self, Name: &str) -> Result<Cow<'static, str>> {
        self.Manager.Get_environment_variable(self.Identifier, Name)
    }

    pub fn Set_environment_variable(&self, Name: &str, Value: &str) -> Result<()> {
        self.Manager
            .Set_environment_variable(self.Identifier, Name, Value)
    }

    pub fn Remove_environment_variable(&self, Name: &str) -> Result<()> {
        self.Manager
            .Remove_environment_variable(self.Identifier, Name)
    }
}
