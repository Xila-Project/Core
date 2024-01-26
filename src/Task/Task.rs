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

    pub fn Get_identifier(&self) -> Task_identifier_type {
        self.Identifier
    }

    pub fn Get_manager(&self) -> &'a Manager_type {
        self.Manager
    }

    pub fn Get_current_task(Manager: &'a Manager_type) -> Result<Self, ()> {
        let Current_task_identifier = Manager.Get_current_task_identifier()?;
        Ok(Self::New(Current_task_identifier, Manager))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test() {
        let Manager = Manager_type::New();

        let Manager_copy = Manager.clone();

        let Root_task = Manager.New_root_task(None, move || {
            let Task = Task_type::Get_current_task(&Manager_copy).unwrap();

            let Child_task = Task
                .New_child_task("Child task", None, || {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                })
                .unwrap();
        });
    }
}
