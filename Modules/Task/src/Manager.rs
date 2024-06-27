// - Dependencies
// - - Local
use super::*;
// - - External
// - - - Standard library
use std::{
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

    fn Get_new_task_identifier(&self) -> Task_identifier_type {
        if self.Tasks.read().unwrap().len() == 0 {
            return Self::Root_task_identifier;
        }

        for Process_identifier in 0..Task_identifier_type::MAX - 1 {
            if !self.Tasks.read().unwrap().contains_key(&Process_identifier) {
                return Process_identifier;
            }
        }
        panic!("No more process identifier available."); // Should never happen since the maximum number of tasks is usize::MAX - 1 which is a lot.
    }

    pub fn Get_task_name(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result<String, Error_type> {
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

        Ok(Child_task_identifier)
    }

    pub fn Get_owner(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result<User_identifier_type, Error_type> {
        let Tasks = self.Tasks.read().unwrap();

        Ok(Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Owner)
    }

    fn Delete_task(&self, Task_identifier: Task_identifier_type) -> Result<(), ()> {
        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock

        // - Remove task from hashmap and take ownership of it
        let Task = match Tasks.remove(&Task_identifier) {
            Some(Task) => Task,
            None => return Err(()),
        };

        std::mem::drop(Tasks); // Force Release lock    // TODO : Find a better way to do this

        // - Waiting for thread to terminate
        for Thread in Task.Threads.into_iter() {
            Thread.Join().unwrap();
        }

        let mut R = Ok(());

        for Child_task_identifier in Task.Children.iter() {
            if self.Delete_task(*Child_task_identifier).is_err() {
                R = Err(());
            }
        }

        R
    }

    pub fn Get_current_task_identifier(&self) -> Result<Task_identifier_type, Error_type> {
        let Tasks = self.Tasks.read().unwrap(); // Acquire lock

        for (Task_identifier, Task) in Tasks.iter() {
            if Task.Thread.Get_identifier() == std::thread::current().id() {
                return Ok(*Task_identifier);
            }
        }

        Err(Error_type::No_thread_for_task)
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
