// - Dependencies
// - - Local
use super::*;
// - - External
// - - - Standard library
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
};
use Users::User_identifier_type;

/// Internal representation of a task.
struct Task_internal_type {
    /// The thread that runs the task.
    Thread: Thread_wrapper_type,
    /// The identifiers of the children of the task.
    Children: Vec<Task_identifier_type>,

    Owner: User_identifier_type,
    /// Environment variables of the task.
    Environment_variables: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

/// A manager for tasks.
#[derive(Clone)]
pub struct Manager_type {
    /// A map of all tasks managed by the manager.
    Tasks: Arc<RwLock<HashMap<Task_identifier_type, Task_internal_type>>>,
}

impl Manager_type {
    /// The identifier of the root task (the task that is created when the manager is created).
    const Root_task_identifier: Task_identifier_type = 0;

    pub fn New() -> Self {
        Manager_type {
            Tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn Get_new_task_identifier(&self) -> Option<Task_identifier_type> {
        if self.Tasks.read().unwrap().len() == 0 {
            return Some(Self::Root_task_identifier);
        }

        (0..Task_identifier_type::MAX)
            .find(|Identifier| !self.Tasks.read().unwrap().contains_key(Identifier))
    }

    pub fn Get_task_name(&self, Task_identifier: Task_identifier_type) -> Result<String> {
        match self.Tasks.read().unwrap().get(&Task_identifier) {
            Some(Task) => match Task.Thread.Get_name() {
                Some(Name) => Ok(Name.to_string()),
                None => Err(Error_type::Invalid_task_identifier),
            },
            None => Err(Error_type::Invalid_task_identifier),
        }
    }

    /// Create a new child task, returns the identifier of the child task.
    /// # Arguments
    /// * `Parent_task_identifier` - The identifier of the parent task.
    /// * `Name` - The human readable name of the task.
    /// * `Stack_size` - The size of the stack of the task.
    /// * `Function` - The function that the task will execute.
    ///
    pub fn New_task<T, F>(
        &self,
        Parent_task_identifier: Option<Task_identifier_type>,
        User_identifier: Option<User_identifier_type>,
        Name: &str,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result<(Task_identifier_type, Join_handle_type<T>)>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let Child_task_identifier = self
            .Get_new_task_identifier()
            .ok_or(Error_type::Too_many_tasks)?;

        // - Create the root task if it's the first task
        let (Owner, Environment_variables) = if self.Tasks.read()?.is_empty() {
            let Owner = match User_identifier {
                Some(Identifier) => Identifier,
                None => Root_user_identifier,
            };
            let Environment_variable = HashMap::new();
            (Owner, Environment_variable)
        }
        // - Create a child task
        else {
            let Parent_task_identifier =
                if let Some(Parent_task_identifier) = Parent_task_identifier {
                    Parent_task_identifier
                } else {
                    self.Get_current_task_identifier()
                        .unwrap_or(Self::Root_task_identifier)
                };

            let mut Tasks = self.Tasks.write()?;

            let Parent_task = Tasks
                .get_mut(&Parent_task_identifier)
                .ok_or(Error_type::Invalid_task_identifier)?;

            Parent_task.Children.push(Child_task_identifier);

            let Owner = match User_identifier {
                Some(Identifier) => Identifier,
                None => self.Get_owner(Parent_task_identifier).unwrap(),
            };

            let Environment_variable = Parent_task.Environment_variables.clone();

            (Owner, Environment_variable)
        };

        let Manager = self.clone();

        let Function = move || {
            let Result = Function();
            let _ = Manager.Delete_task(Child_task_identifier);
            Result
        };

        let Join_handle = Thread_wrapper_type::New(Name, Stack_size, Function)?;

        let Thread = Join_handle.Get_thread_wrapper();

        self.Tasks.write()?.insert(
            Child_task_identifier,
            Task_internal_type {
                Thread,
                Children: Vec::new(),
                Owner,
                Environment_variables,
            },
        );

        Ok((Child_task_identifier, Join_handle))
    }

    pub fn Get_owner(&self, Task_identifier: Task_identifier_type) -> Result<User_identifier_type> {
        let Tasks = self.Tasks.read().unwrap();

        Ok(Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Owner)
    }

    fn Delete_task(&self, Task_identifier: Task_identifier_type) -> Result<()> {
        // - Wait for all children to terminate
        while !self
            .Tasks
            .read()?
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Children
            .is_empty()
        {
            Task_type::Sleep(std::time::Duration::from_millis(10));
        }

        self.Tasks
            .write()?
            .remove(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?;

        Ok(())
    }

    pub fn Get_current_task_identifier(&self) -> Result<Task_identifier_type> {
        let Tasks = self.Tasks.read().unwrap(); // Acquire lock

        for (Task_identifier, Task) in Tasks.iter() {
            if Task.Thread.Get_identifier() == std::thread::current().id() {
                return Ok(*Task_identifier);
            }
        }

        Err(Error_type::No_thread_for_task)
    }

    pub fn Get_current_task(&self) -> Result<Task_type> {
        Ok(Task_type::New(
            self.Get_current_task_identifier()?,
            self.clone(),
        ))
    }

    pub fn Get_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result<Cow<'static, str>> {
        let Tasks = self.Tasks.read().unwrap(); // Acquire lock

        Ok(Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .get(Name)
            .ok_or(Error_type::Invalid_environment_variable)?
            .clone())
    }

    pub fn Get_environment_variables(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result<HashMap<Cow<'static, str>, Cow<'static, str>>> {
        let Tasks = self.Tasks.read().unwrap(); // Acquire lock

        Ok(Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .clone())
    }

    pub fn Set_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
        Value: &str,
    ) -> Result<()> {
        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock

        Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .insert(Cow::Owned(Name.to_string()), Cow::Owned(Value.to_string()));

        Ok(())
    }

    pub fn Remove_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result<()> {
        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock

        Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .remove(Name);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn Test() {
        let Manager = Manager_type::New();

        Manager.clone().New_root_task(None, move || {
            let _ = Manager
                .New_task(
                    Manager_type::Root_task_identifier,
                    None,
                    "Child task",
                    None,
                    || {
                        Task_type::Sleep(Duration::from_millis(100));
                    },
                )
                .unwrap();

            Manager
                .Delete_task(Manager_type::Root_task_identifier)
                .unwrap();
            Task_type::Sleep(Duration::from_millis(100));
        });
    }
}
