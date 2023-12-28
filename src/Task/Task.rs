use super::*;

pub type Task_identifier_type = usize;

/// A wrapper for individual tasks that are managed by [Manager_type].
pub struct Task_type<'a> {
    /// The identifier of the task.
    Identifier: Task_identifier_type,
    /// A reference to the [Manager_type] that manages the task.
    Manager: &'a Manager_type,
}

impl<'a> Task_type<'a> {
    fn New(Identifier: Task_identifier_type, Manager: &'a Manager_type) -> Self {
        Self {
            Identifier,
            Manager,
        }
    }

    pub fn New_child_task<F>(
        &self,
        Name: &str,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result<Task_type, ()>
    where
        F: FnOnce() + Send + 'static,
    {
        match self
            .Manager
            .New_task(self.Identifier, Name, Stack_size, Function)
        {
            Ok(Child_task_identifier) => Ok(Self::New(Child_task_identifier, self.Manager)),
            Err(()) => Err(()),
        }
    }

    pub fn Get_name(&self) -> Result<String, ()> {
        self.Manager.Get_task_name(self.Identifier)
    }
}
