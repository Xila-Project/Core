// - Dependencies
// - - Local
use super::*;
// - - External
// - - - Standard library
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Internal representation of a task.
struct Task_internal_type {
    Thread: Thread_wrapper_type,
    Children: Vec<Task_identifier_type>,
}

/// A manager for tasks.
pub struct Manager_type {
    /// A map of all tasks managed by the manager.
    Tasks: Arc<RwLock<HashMap<Task_identifier_type, Task_internal_type>>>,
}

impl Manager_type {
    /// The identifier of the root task (the task that is created when the manager is created).
    const Root_task_identifier: Task_identifier_type = 0;

    pub fn New<F>(Main_task_function: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let Manager = Manager_type {
            Tasks: Arc::new(RwLock::new(HashMap::new())),
        };

        if Manager
            .New_task(Self::Root_task_identifier, "Xila", None, Main_task_function)
            .is_err()
        {
            panic!("Failed to create root task."); // If this happens, crazy shits are going on, so panic.
        }

        Manager
    }

    fn Get_new_task_identifier(&self) -> Task_identifier_type {
        if self.Tasks.read().unwrap().len() == 0 {
            return Self::Root_task_identifier;
        }

        for Process_identifier in 0..std::usize::MAX - 1 {
            if !self.Tasks.read().unwrap().contains_key(&Process_identifier) {
                return Process_identifier;
            }
        }
        panic!("No more process identifier available."); // Should never happen since the maximum number of tasks is usize::MAX - 1 which is a lot.
    }

    pub fn Get_task_name(&self, Process_identifier: Task_identifier_type) -> Result<String, ()> {
        match self.Tasks.read().unwrap().get(&Process_identifier) {
            Some(Task) => match Task.Thread.Get_name() {
                Some(Name) => Ok(Name.to_string()),
                None => Err(()),
            },
            None => Err(()),
        }
    }

    /// Create a new child task,
    ///
    pub fn New_task<F>(
        &self,
        Parent_task_identifier: Task_identifier_type,
        Name: &str,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result<Task_identifier_type, ()>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock

        let Parent_task = match Tasks.get_mut(&Parent_task_identifier) {
            Some(Parent_task) => Parent_task,
            None => return Err(()),
        };

        let Thread_wrapper = match Thread_wrapper_type::New(Name, Stack_size, Function) {
            Ok(Thread_wrapper) => Thread_wrapper,
            Err(()) => return Err(()),
        };

        let Child_task_identifier = self.Get_new_task_identifier();

        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock again

        Tasks.insert(
            Child_task_identifier,
            Task_internal_type {
                Thread: Thread_wrapper,
                Children: Vec::new(),
            },
        );

        Parent_task.Children.push(Child_task_identifier);

        Ok(Child_task_identifier)
    }
}
