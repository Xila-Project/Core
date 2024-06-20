use super::*;
use Users::User_identifier_type;

pub type Task_identifier_type = usize;

/// A wrapper for individual tasks that are managed by [Manager_type].
pub struct Task_type<'a> {
    /// The identifier of the task.
    Identifier: Task_identifier_type,
    /// A reference to the [Manager_type] that manages the task.
    Manager: &'a Manager_type,
}

impl<'a> Task_type<'a> {
    /// Internal method to create a new task.
    fn New(Identifier: Task_identifier_type, Manager: &'a Manager_type) -> Self {
        Self {
            Identifier,
            Manager,
        }
    }

    /// Create a new child task.
    pub fn New_child_task<F>(
        &self,
        Name: &str,
        Owner: Option<User_identifier_type>,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result<Task_type, Error_type>
    where
        F: FnOnce() + Send + 'static,
    {
        Ok(Self::New(
            self.Manager
                .New_task(self.Identifier, Owner, Name, Stack_size, Function)?,
            self.Manager,
        ))
    }

    pub fn Get_name(&self) -> Result<String, Error_type> {
        self.Manager.Get_task_name(self.Identifier)
    }

    pub fn Get_identifier(&self) -> Task_identifier_type {
        self.Identifier
    }

    pub fn Get_manager(&self) -> &'a Manager_type {
        self.Manager
    }

    pub fn Get_owner(&self) -> Result<User_identifier_type, Error_type> {
        self.Manager.Get_owner(self.Identifier)
    }

    pub fn Get_current_task(Manager: &'a Manager_type) -> Result<Self, Error_type> {
        let Current_task_identifier = Manager.Get_current_task_identifier()?;
        Ok(Self::New(Current_task_identifier, Manager))
    }

    pub fn Sleep(Duration: std::time::Duration) {
        Thread_wrapper_type::Sleep(Duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test() {
        let Manager = Manager_type::New();

        let Manager_copy = Manager.clone();

        Manager.New_root_task(None, move || {
            let Task = Task_type::Get_current_task(&Manager_copy).unwrap();

            let _ = Task
                .New_child_task("Child task", None, None, || {
                    Task_type::Sleep(std::time::Duration::from_millis(100));
                })
                .unwrap();
        });
    }
}
