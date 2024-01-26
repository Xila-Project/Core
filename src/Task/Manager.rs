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
    /// The thread that runs the task.
    Thread: Thread_wrapper_type,
    /// The identifiers of the children of the task.
    Children: Vec<Task_identifier_type>,
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
        let Manager = Manager_type {
            Tasks: Arc::new(RwLock::new(HashMap::new())),
        };

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

    pub fn New_root_task<F>(&self, Stack_size: Option<usize>, Function: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let Thread_wrapper = match Thread_wrapper_type::New("Xila", Stack_size, Function) {
            Ok(Thread_wrapper) => Thread_wrapper,
            Err(()) => panic!(),
        };

        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock

        Tasks.insert(
            Self::Root_task_identifier,
            Task_internal_type {
                Thread: Thread_wrapper,
                Children: Vec::new(),
            },
        );
    }

    /// Create a new child task, returns the identifier of the child task.
    /// # Arguments
    /// * `Parent_task_identifier` - The identifier of the parent task.
    /// * `Name` - The human readable name of the task.
    /// * `Stack_size` - The size of the stack of the task.
    /// * `Function` - The function that the task will execute.
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
        let Child_task_identifier = self.Get_new_task_identifier();

        let mut Tasks = self.Tasks.write().unwrap(); // Acquire lock

        let Parent_task = match Tasks.get_mut(&Parent_task_identifier) {
            Some(Parent_task) => Parent_task,
            None => return Err(()),
        };

        let Thread_wrapper = match Thread_wrapper_type::New(Name, Stack_size, Function) {
            Ok(Thread_wrapper) => Thread_wrapper,
            Err(()) => return Err(()),
        };

        Parent_task.Children.push(Child_task_identifier);

        std::mem::drop(Tasks); // Force Release lock    // TODO : Find a better way to do this
        self.Tasks.write().unwrap().insert(
            Child_task_identifier,
            Task_internal_type {
                Thread: Thread_wrapper,
                Children: Vec::new(),
            },
        );

        Ok(Child_task_identifier)
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
        Task.Thread.Join().unwrap();

        let mut R = Ok(());

        for Child_task_identifier in Task.Children.iter() {
            if self.Delete_task(*Child_task_identifier).is_err() {
                R = Err(());
            }
        }

        R
    }

    pub fn Get_current_task_identifier(&self) -> Result<Task_identifier_type, ()> {
        let Tasks = self.Tasks.read().unwrap(); // Acquire lock

        for (Task_identifier, Task) in Tasks.iter() {
            if Task.Thread.Get_name().unwrap() == std::thread::current().name().unwrap() {
                return Ok(*Task_identifier);
            }
        }

        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    fn Test() {
        let Manager = Manager_type::New();

        Manager.New_root_task(None, || {
            Task_type::Sleep(Duration::from_millis(100));
        });

        let _ = Manager
            .New_task(
                Manager_type::Root_task_identifier,
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
    }
}
