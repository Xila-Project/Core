// - Dependencies
// - - Local
use super::*;
// - - External
// - - - Standard library
use std::{
    collections::BTreeMap,
    sync::{OnceLock, RwLock},
    time::Duration,
};
use Users::User_identifier_type;

/// Internal representation of a task.
struct Task_internal_type {
    /// The thread that runs the task.
    Main_thread: Thread_wrapper_type,
    /// The children of the task.
    Parent: Task_identifier_type,
    /// The identifier of the user that owns the task.
    Owner: User_identifier_type,
    /// Environment variables of the task.
    Environment_variables: Vec<Environment_variable_type>,
}

static Manager_instance: OnceLock<Manager_type> = OnceLock::new();

pub fn Initialize() -> Result_type<&'static Manager_type> {
    Manager_instance.get_or_init(Manager_type::New);

    Ok(Get_instance())
}

pub fn Get_instance() -> &'static Manager_type {
    Manager_instance
        .get()
        .expect("Cannot get Task manager instance before initialization")
}

pub fn Is_initialized() -> bool {
    Manager_instance.get().is_some()
}

struct Inner_manager_type {
    /// A map of all tasks managed by the Get_instance().unwrap().
    Tasks: BTreeMap<Task_identifier_type, Task_internal_type>,
    /// A map of all threads and their parent task.
    Threads: BTreeMap<Thread_identifier_type, Task_identifier_type>,
}

/// A manager for tasks.
pub struct Manager_type(RwLock<Inner_manager_type>);

impl Manager_type {
    pub const Root_task_identifier: Task_identifier_type = Task_identifier_type::New(0);

    /// Create a new task manager instance,
    /// create a root task and register current thread as the root task main thread.
    fn New() -> Self {
        let Manager = Manager_type(RwLock::new(Inner_manager_type {
            Tasks: BTreeMap::new(),
            Threads: BTreeMap::new(),
        }));

        let mut Inner = Manager.0.write().expect("Failed to acquire write lock");

        // Create root task which is its own parent
        let Task_identifier = Self::Root_task_identifier;
        let Task_internal = Task_internal_type {
            Main_thread: Thread_wrapper_type::Get_current(),
            Parent: Task_identifier,
            Owner: User_identifier_type::Root,
            Environment_variables: vec![],
        };

        Self::Register_task_internal(Task_identifier, Task_internal, &mut Inner.Tasks)
            .expect("Failed to register root task");

        drop(Inner); // Release write lock

        // Add current thread to tasks as root
        let Thread_identifier = Manager.Get_current_thread_identifier();
        Manager
            .Register_thread(Task_identifier, Thread_identifier)
            .expect("Failed to register root thread");

        Manager
    }

    /// Register the current thread as a task.
    ///
    /// This function should ONLY be called for testing purposes.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it can lead to undefined behavior if the current thread is already registered.
    pub unsafe fn Register_task(&self) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        let Task_identifier = Self::Get_new_task_identifier(&Inner.Tasks)?;

        // Create root task which is its own parent
        let Task_internal = Task_internal_type {
            Main_thread: Thread_wrapper_type::Get_current(),
            Parent: Self::Root_task_identifier,
            Owner: User_identifier_type::Root,
            Environment_variables: vec![],
        };

        if Inner
            .Threads
            .insert(
                Thread_wrapper_type::Get_current().Get_identifier(),
                Task_identifier,
            )
            .is_some()
        {
            return Err(Error_type::Thread_already_registered);
        }

        Self::Register_task_internal(Task_identifier, Task_internal, &mut Inner.Tasks)
    }

    fn Get_new_task_identifier(
        Tasks: &BTreeMap<Task_identifier_type, Task_internal_type>,
    ) -> Result_type<Task_identifier_type> {
        (0..Task_identifier_type::Maximum)
            .map(Task_identifier_type::from)
            .find(|Identifier| !Tasks.contains_key(Identifier))
            .ok_or(Error_type::Too_many_tasks)
    }

    pub fn Get_thread_name(&self) -> Option<String> {
        Some(Thread_wrapper_type::Get_current().Get_name()?.to_owned())
    }

    /// # Arguments
    /// * `Task_identifier` - The identifier of the task.
    pub fn Get_task_name(&self, Task_identifier: Task_identifier_type) -> Result_type<String> {
        Ok(self
            .0
            .read()?
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Main_thread
            .Get_name()
            .ok_or(Error_type::Invalid_task_identifier)?
            .to_string())
    }

    /// Register a task with its parent task.
    ///
    /// This function check if the task identifier is not already used,
    /// however it doesn't check if the parent task exists.
    fn Register_task_internal(
        Task_identifier: Task_identifier_type,
        Task_internal: Task_internal_type,
        Tasks: &mut BTreeMap<Task_identifier_type, Task_internal_type>,
    ) -> Result_type<()> {
        if Tasks.insert(Task_identifier, Task_internal).is_some() {
            return Err(Error_type::Invalid_task_identifier);
        }

        Ok(())
    }

    fn Register_thread(
        &self,
        Task_identifier: Task_identifier_type,
        Thread_identifier: Thread_identifier_type,
    ) -> Result_type<()> {
        self.0
            .write()?
            .Threads
            .insert(Thread_identifier, Task_identifier);

        Ok(())
    }

    fn Unregister_thread(
        &self,
        Task_identifier: Task_identifier_type,
        Thread_identifier: Thread_identifier_type,
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        // Remove the thread
        if Inner.Threads.remove(&Thread_identifier).is_none() {
            return Err(Error_type::Thread_not_registered);
        }

        // If this is the task main thread
        if Inner
            .Tasks
            .get(&Task_identifier)
            .unwrap()
            .Main_thread
            .Get_identifier()
            == Thread_identifier
        {
            // Unregister the task
            Self::Unregister_task(Task_identifier, &mut Inner)?;
        }

        Ok(())
    }

    pub fn New_thread_internal<T, F>(
        Parent_task_identifier: Task_identifier_type,
        Name: &str,
        Stack_size: Option<usize>,
        Function: F,
        Main_thread: bool,
    ) -> Result_type<Join_handle_type<T>>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let Closure = move || {
            let Thread_identifier = Get_instance().Get_current_thread_identifier();

            if Main_thread {
                // Wait for the task to be registered
                while !Get_instance()
                    .0
                    .read()
                    .expect("Failed to acquire read lock")
                    .Tasks
                    .contains_key(&Parent_task_identifier)
                {
                    Self::Sleep(Duration::from_millis(10));
                }
            }

            // The thread registers itself
            Get_instance()
                .Register_thread(Parent_task_identifier, Thread_identifier)
                .expect("Failed to register thread");

            let Result = Function();

            // The thread unregister itself upon termination
            Get_instance()
                .Unregister_thread(Parent_task_identifier, Thread_identifier)
                .expect("Failed to unregister thread");

            Result
        };

        let Join_handle = Thread_wrapper_type::Spawn(Name, Stack_size, Closure)?;

        Ok(Join_handle)
    }

    /// Create a new thread, returns the identifier of the thread.
    ///
    /// This function checks if the parent task exists.
    pub fn New_thread<T, F>(
        &self,
        Parent_task_identifier: Task_identifier_type,
        Name: &str,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result_type<Join_handle_type<T>>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        // Check if the parent task exists
        if !self.0.read()?.Tasks.contains_key(&Parent_task_identifier) {
            return Err(Error_type::Invalid_task_identifier);
        }

        Self::New_thread_internal(Parent_task_identifier, Name, Stack_size, Function, false)
    }

    /// Sleep the current thread for a given duration.
    pub fn Sleep(Duration: std::time::Duration) {
        Thread_wrapper_type::Sleep(Duration);
    }

    /// Create a new child task, returns the identifier of the child task.
    /// # Arguments
    /// * `Parent_task_identifier` - The identifier of the parent task, if None, the current task is used.
    /// * `Owner` - The identifier of the user that owns the task, if None, the parent task owner is used.
    /// * `Name` - The human readable name of the task.
    /// * `Stack_size` - The size of the stack of the task.
    /// * `Function` - The function that the task will execute.
    ///
    pub fn New_task<T, F>(
        &self,
        Parent_task_identifier: Task_identifier_type,
        Owner: Option<User_identifier_type>,
        Name: &str,
        Stack_size: Option<usize>,
        Function: F,
    ) -> Result_type<(Task_identifier_type, Join_handle_type<T>)>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let Inner = self.0.read()?;

        let (Owner, Environment_variables) = {
            let Parent_task = Inner
                .Tasks
                .get(&Parent_task_identifier)
                .ok_or(Error_type::Invalid_task_identifier)?;

            let Owner = Owner.unwrap_or(Parent_task.Owner);

            (Owner, Parent_task.Environment_variables.clone())
        };

        let Child_task_identifier = Self::Get_new_task_identifier(&Inner.Tasks)?;

        drop(Inner); // Release read lock to avoid deadlock

        // Create a new thread for the task
        let Join_handle =
            Self::New_thread_internal(Child_task_identifier, Name, Stack_size, Function, true)?;

        Self::Register_task_internal(
            Child_task_identifier,
            Task_internal_type {
                Main_thread: Join_handle.Get_thread_wrapper(),
                Parent: Parent_task_identifier,
                Owner,
                Environment_variables,
            },
            &mut self.0.write()?.Tasks,
        )?;

        Ok((Child_task_identifier, Join_handle))
    }

    /// Get the children tasks of a task.
    pub fn Get_child_tasks(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Task_identifier_type>> {
        Ok(self
            .0
            .read()?
            .Tasks
            .iter()
            .filter(|(_, Task)| Task.Parent == Task_identifier)
            .map(|(Identifier, _)| *Identifier)
            .collect())
    }

    /// Get the parent task of a task.
    pub fn Get_parent_task(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Task_identifier_type> {
        Ok(self
            .0
            .read()?
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Parent)
    }

    /// Get the children threads of a task.
    pub fn Get_children_threads(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Thread_identifier_type>> {
        Ok(self
            .0
            .read()?
            .Threads
            .iter()
            .filter(|(_, Parent)| **Parent == Task_identifier)
            .map(|(Identifier, _)| *Identifier)
            .collect())
    }

    /// Get user identifier of the owner of a task.
    pub fn Get_user(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<User_identifier_type> {
        Ok(self
            .0
            .read()?
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Owner)
    }

    /// Unregister task.
    ///
    /// If the task has children thread, wait for them to terminate.
    /// If the task has children tasks, the root task adopts them.
    fn Unregister_task(
        Task_identifier: Task_identifier_type,
        Inner: &mut Inner_manager_type,
    ) -> Result_type<()> {
        // - Wait for all child threads to terminate
        while Inner
            .Threads
            .iter()
            .any(|(_, Task)| *Task == Task_identifier)
        {
            Self::Sleep(Duration::from_millis(100));
        }

        // - Root task adopts all children of the task
        Inner.Tasks.iter_mut().for_each(|(_, Task)| {
            if Task.Parent == Task_identifier {
                Task.Parent = Self::Root_task_identifier;
            }
        });

        // - Remove the task
        if Inner.Tasks.remove(&Task_identifier).is_none() {
            return Err(Error_type::Invalid_task_identifier);
        }

        Ok(())
    }

    pub fn Get_current_thread_identifier(&self) -> Thread_identifier_type {
        Thread_wrapper_type::Get_current().Get_identifier()
    }

    pub fn Get_current_task_identifier(&self) -> Result_type<Task_identifier_type> {
        let Current_thread = self.Get_current_thread_identifier();

        self.0
            .read()?
            .Threads
            .get(&Current_thread)
            .cloned()
            .ok_or(Error_type::Thread_not_registered)
    }

    pub fn Get_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result_type<Environment_variable_type> {
        Ok(self
            .0
            .read()?
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .iter()
            .find(|Variable| Variable.Get_name() == Name)
            .ok_or(Error_type::Invalid_environment_variable)?
            .clone())
    }

    pub fn Get_environment_variables(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Environment_variable_type>> {
        Ok(self
            .0
            .read()?
            .Tasks
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
    ) -> Result_type<()> {
        let Environment_variable = Environment_variable_type::New(Name, Value);

        self.0
            .write()?
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .push(Environment_variable);

        Ok(())
    }

    pub fn Remove_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result_type<()> {
        self.0
            .write()?
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .retain(|Variable| Variable.Get_name() != Name);

        Ok(())
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_task_manager() {
        let Manager = Initialize().expect("Failed to initialize task manager");

        // Run test sequentially since the instance is shared

        println!("Run test : Test_get_task_name");
        Test_get_task_name(Manager);
        println!("Run test : Test_new_task");
        Test_new_task(Manager);
        println!("Run test : Test_get_owner");
        Test_get_owner(Manager);
        println!("Run test : Test_get_current_task_identifier");
        Test_get_current_task_identifier(Manager);
        println!("Run test : Test_task_owner_inheritance");
        Test_task_owner_inheritance(Manager);
        println!("Run test : Test_environment_variables");
        Test_environment_variables(Manager);
        println!("Run test : Test_environment_variable_inheritance");
        Test_environment_variable_inheritance(Manager);
        println!("Run test : Test_join_handle");
        Test_join_handle(Manager);
    }

    fn Test_get_task_name(Manager: &Manager_type) {
        let Task_name = "Test Task";
        let Task = Manager.Get_current_task_identifier().unwrap();

        let _ = Manager
            .New_task(Task, None, Task_name, None, move || {
                let Task = Get_instance().Get_current_task_identifier().unwrap();

                assert_eq!(Get_instance().Get_task_name(Task).unwrap(), Task_name);
            })
            .unwrap();
    }

    fn Test_new_task(Manager: &Manager_type) {
        let Task_name = "Child Task";
        let Task = Manager.Get_current_task_identifier().unwrap();

        let _ = Manager
            .New_task(Task, None, Task_name, None, || {})
            .unwrap();
    }

    fn Test_get_owner(Manager: &Manager_type) {
        let User_identifier = User_identifier_type::New(123); // Assuming User_identifier_type is i32 for example

        let Task = Manager.Get_current_task_identifier().unwrap();

        let _ = Manager
            .New_task(
                Task,
                Some(User_identifier),
                "Task with owner",
                None,
                move || {
                    let Task = Get_instance().Get_current_task_identifier().unwrap();

                    assert_eq!(Get_instance().Get_user(Task).unwrap(), User_identifier);
                },
            )
            .unwrap();
    }

    fn Test_get_current_task_identifier(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();

        let (_, Join_handle) = Manager
            .New_task(Task, None, "Current Task", None, move || {
                let _ = Get_instance().Get_current_task_identifier().unwrap();
            })
            .unwrap();

        Join_handle.Join().unwrap();
    }

    fn Test_task_owner_inheritance(Manager: &Manager_type) {
        let User_identifier = User_identifier_type::New(123); // Assuming User_identifier_type is i32 for example
        let Task = Manager.Get_current_task_identifier().unwrap();

        let _ = Manager
            .New_task(Task, Some(User_identifier), "Task 1", None, move || {
                let Task = Get_instance().Get_current_task_identifier().unwrap();

                assert_eq!(Get_instance().Get_user(Task).unwrap(), User_identifier);

                // - Inherit owner
                let _ = Get_instance()
                    .New_task(Task, None, "Task 2", None, move || {
                        let Task = Get_instance().Get_current_task_identifier().unwrap();

                        assert_eq!(Get_instance().Get_user(Task).unwrap(), User_identifier);
                        assert_eq!(Get_instance().Get_task_name(Task).unwrap(), "Task 2");

                        Manager_type::Sleep(std::time::Duration::from_secs(1));
                    })
                    .unwrap();

                let User_identifier = User_identifier_type::New(6969); // Assuming User_identifier_type is i32 for example

                // - Overwrite owner
                let _ = Get_instance()
                    .New_task(Task, Some(User_identifier), "Task 3", None, move || {
                        let Task = Get_instance().Get_current_task_identifier().unwrap();

                        assert_eq!(Get_instance().Get_user(Task).unwrap(), User_identifier);
                        assert_eq!(Get_instance().Get_task_name(Task).unwrap(), "Task 3");
                    })
                    .unwrap();
            })
            .unwrap();
    }

    fn Test_environment_variables(Manager: &Manager_type) {
        let Task_identifier = Manager.Get_current_task_identifier().unwrap();
        let Name = "Key";
        let Value = "Value";

        Manager
            .Set_environment_variable(Task_identifier, Name, Value)
            .unwrap();
        assert_eq!(
            Manager
                .Get_environment_variable(Task_identifier, Name)
                .unwrap()
                .Get_value(),
            Value
        );
        Manager
            .Remove_environment_variable(Task_identifier, Name)
            .unwrap();
        assert!(Manager
            .Get_environment_variable(Task_identifier, Name)
            .is_err());
    }

    fn Test_environment_variable_inheritance(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();

        let _ = Manager
            .New_task(Task, None, "Child Task", None, move || {
                let Current_task = Get_instance().Get_current_task_identifier().unwrap();

                Get_instance()
                    .Set_environment_variable(Task, "Key", "Value")
                    .unwrap();

                let _ = Get_instance()
                    .New_task(Current_task, None, "Grandchild Task", None, || {
                        let Current_task = Get_instance().Get_current_task_identifier().unwrap();

                        assert_eq!(
                            Get_instance()
                                .Get_environment_variable(Current_task, "Key")
                                .unwrap()
                                .Get_value(),
                            "Value"
                        );
                    })
                    .unwrap();
            })
            .unwrap();
    }

    fn Test_join_handle(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();

        let (_, Join_handle) = Manager
            .New_task(Task, None, "Task with join handle", None, || 42)
            .unwrap();
        let Result = Join_handle.Join();
        assert_eq!(Result.unwrap(), 42);
    }
}
