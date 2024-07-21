// - Dependencies
// - - Local
use super::*;
// - - External
// - - - Standard library
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    sync::RwLock,
};
use Users::{Root_user_identifier, User_identifier_type};

/// Internal representation of a task.
struct Task_internal_type {
    /// The thread that runs the task.
    Thread: Thread_wrapper_type,
    /// The identifiers of the children of the task.
    Children: Vec<Task_identifier_type>,
    /// The identifier of the user that owns the task.
    Owner: User_identifier_type,
    /// Environment variables of the task.
    Environment_variables: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

static mut Manager_instance: Option<Manager_type> = None;

pub fn Initialize() -> Result_type<&'static Manager_type> {
    if Is_initialized() {
        return Err(Error_type::Already_initialized);
    }

    unsafe {
        Manager_instance = Some(Manager_type::New());
    }

    Get_instance()
}

pub fn Get_instance() -> Result_type<&'static Manager_type> {
    unsafe { Manager_instance.as_ref().ok_or(Error_type::Not_initialized) }
}

pub fn Is_initialized() -> bool {
    unsafe { Manager_instance.is_some() }
}

/// A manager for tasks.
pub struct Manager_type {
    /// A map of all tasks managed by the Get_instance().unwrap().
    Tasks: RwLock<BTreeMap<Task_identifier_type, Task_internal_type>>,
}

impl Manager_type {
    fn New() -> Self {
        // Add current thread to tasks as root task
        let Task_internal = Task_internal_type {
            Thread: Thread_wrapper_type::Get_current(),
            Children: Vec::new(),
            Owner: Root_user_identifier,
            Environment_variables: HashMap::new(),
        };

        let mut Tasks_map = BTreeMap::new();

        Tasks_map.insert(Task_identifier_type::from(0), Task_internal);

        Manager_type {
            Tasks: RwLock::new(Tasks_map),
        }
    }

    fn Get_new_task_identifier(&self) -> Option<Task_identifier_type> {
        (0..Task_identifier_type::Maximum)
            .find(|Identifier| {
                !self
                    .Tasks
                    .read()
                    .unwrap()
                    .contains_key(&(*Identifier).into())
            })
            .map(|Identifier| Identifier.into())
    }

    pub fn Get_task_name(&self, Task_identifier: Task_identifier_type) -> Result_type<String> {
        match self.Tasks.read()?.get(&Task_identifier) {
            Some(Task) => match Task.Thread.Get_name() {
                Some(Name) => Ok(Name.to_string()),
                None => Err(Error_type::Invalid_task_identifier),
            },
            None => Err(Error_type::Invalid_task_identifier),
        }
    }

    /// Create a new child task, returns the identifier of the child task.
    /// # Arguments
    /// * `Parent_task_identifier` - The identifier of the parent task, if None, the current task is used.
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
    ) -> Result_type<(Task_identifier_type, Join_handle_type<T>)>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let Child_task_identifier = self
            .Get_new_task_identifier()
            .ok_or(Error_type::Too_many_tasks)?;

        let Parent_task_identifier = if let Some(Parent_task_identifier) = Parent_task_identifier {
            Parent_task_identifier
        } else {
            self.Get_current_task_identifier()?
        };

        let Owner = match User_identifier {
            Some(Identifier) => Identifier,
            None => self.Get_owner(Parent_task_identifier)?,
        };

        let mut Tasks = self.Tasks.write()?;

        let Parent_task = Tasks
            .get_mut(&Parent_task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?;

        Parent_task.Children.push(Child_task_identifier);

        let Environment_variables = Parent_task.Environment_variables.clone();

        let Function = move || {
            let Result = Function();
            let _ = Get_instance()
                .expect("Failed to get instance")
                .Delete_task(Child_task_identifier);
            Result
        };

        let Join_handle = Thread_wrapper_type::New(Name, Stack_size, Function)?;

        let Thread = Join_handle.Get_thread_wrapper();

        Tasks.insert(
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

    pub fn Get_owner(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<User_identifier_type> {
        let Tasks = self.Tasks.read()?;

        Ok(Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Owner)
    }

    fn Delete_task(&self, Task_identifier: Task_identifier_type) -> Result_type<()> {
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

    pub fn Get_current_task_identifier(&self) -> Result_type<Task_identifier_type> {
        let Tasks = self.Tasks.read()?;

        for (Task_identifier, Task) in Tasks.iter() {
            if Task.Thread.Get_identifier() == std::thread::current().id() {
                return Ok(*Task_identifier);
            }
        }

        Err(Error_type::No_thread_for_task)
    }

    pub fn Get_current_task(&self) -> Result_type<Task_type> {
        Ok(Task_type::New(self.Get_current_task_identifier()?))
    }

    pub fn Get_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result_type<Cow<'static, str>> {
        let Tasks = self.Tasks.read()?; // Acquire lock

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
    ) -> Result_type<HashMap<Cow<'static, str>, Cow<'static, str>>> {
        let Tasks = self.Tasks.read()?; // Acquire lock

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
    ) -> Result_type<()> {
        let mut Tasks = self.Tasks.write()?; // Acquire lock

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
    ) -> Result_type<()> {
        let mut Tasks = self.Tasks.write()?; // Acquire lock

        Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .remove(Name);

        Ok(())
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_task_manager() {
        let Manager = Initialize().expect("Failed to initialize task manager");

        Test_get_task_name(Manager);
        Test_new_task(Manager);
        Test_delete_task(Manager);
        Test_get_owner(Manager);
        Test_get_current_task_identifier(Manager);
        Test_multiple_tasks_with_same_owner(Manager);
        Test_environment_variables(Manager);
        Test_environment_variable_inheritance(Manager);
        Test_join_handle(Manager);
    }

    fn Test_get_task_name(Manager: &Manager_type) {
        let Task_name = "Test Task";
        let (Task_identifier, _) = Manager
            .New_task(None, None, Task_name, None, || {})
            .unwrap();
        assert_eq!(
            Get_instance()
                .unwrap()
                .Get_task_name(Task_identifier)
                .unwrap(),
            Task_name
        );
    }

    fn Test_new_task(Manager: &Manager_type) {
        let Task_name = "Child Task";
        let (Task_identifier, _) = Manager
            .New_task(None, None, Task_name, None, || {})
            .unwrap();
        assert!(Manager.Get_task_name(Task_identifier).is_ok());
    }

    fn Test_delete_task(Manager: &Manager_type) {
        let (Task_identifier, _) = Manager
            .New_task(None, None, "Task to delete", None, || {})
            .unwrap();
        assert!(Manager.Delete_task(Task_identifier).is_ok());
        assert!(Manager.Get_task_name(Task_identifier).is_err());
    }

    fn Test_get_owner(Manager: &Manager_type) {
        let User_identifier = 123; // Assuming User_identifier_type is i32 for example
        let (Task_identifier, _) = Manager
            .New_task(None, Some(User_identifier), "Task with owner", None, || {})
            .unwrap();
        assert_eq!(Manager.Get_owner(Task_identifier).unwrap(), User_identifier);
    }

    fn Test_get_current_task_identifier(Manager: &Manager_type) {
        let (Task_identifier, Join_handle) = Manager
            .New_task(None, None, "Current Task", None, move || {
                let _ = Get_instance()
                    .unwrap()
                    .Get_current_task_identifier()
                    .unwrap();
            })
            .unwrap();
        let _ = Get_instance().unwrap().Get_task_name(Task_identifier); // Just to use Task_identifier and avoid unused variable warning.
        Join_handle.Join().unwrap();
    }

    fn Test_multiple_tasks_with_same_owner(Manager: &Manager_type) {
        let User_identifier = 123; // Assuming User_identifier_type is i32 for example
        let (Task_identifier_1, _) = Manager
            .New_task(None, Some(User_identifier), "Task 1", None, move || {
                let (Task_identifier_2, _) = Get_instance()
                    .unwrap()
                    .New_task(None, None, "Task 2", None, move || {
                        assert_eq!(
                            Get_instance()
                                .unwrap()
                                .Get_current_task()
                                .unwrap()
                                .Get_owner()
                                .unwrap(),
                            User_identifier
                        );
                        assert_eq!(
                            Get_instance()
                                .unwrap()
                                .Get_current_task()
                                .unwrap()
                                .Get_name()
                                .unwrap(),
                            "Task 2"
                        );

                        Task_type::Sleep(std::time::Duration::from_secs(1));
                    })
                    .unwrap();

                assert_eq!(
                    Get_instance()
                        .unwrap()
                        .Get_owner(Task_identifier_2)
                        .unwrap(),
                    User_identifier
                );

                let _ = Get_instance()
                    .unwrap()
                    .New_task(None, Some(6969), "Task 3", None, move || {
                        assert_eq!(
                            Get_instance()
                                .unwrap()
                                .Get_current_task()
                                .unwrap()
                                .Get_owner()
                                .unwrap(),
                            6969
                        );
                        assert_eq!(
                            Get_instance()
                                .unwrap()
                                .Get_current_task()
                                .unwrap()
                                .Get_name()
                                .unwrap(),
                            "Task 3"
                        );
                    })
                    .unwrap();
            })
            .unwrap();

        assert_eq!(
            Get_instance()
                .unwrap()
                .Get_owner(Task_identifier_1)
                .unwrap(),
            User_identifier
        );
    }

    fn Test_environment_variables(Manager: &Manager_type) {
        let Task_identifier = Manager
            .Get_current_task_identifier()
            .expect("Failed to get current task identifier");
        let Name = "Key";
        let Value = "Value";
        Manager
            .Set_environment_variable(Task_identifier, Name, Value)
            .unwrap();
        assert_eq!(
            Manager
                .Get_environment_variable(Task_identifier, Name)
                .unwrap(),
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
        Manager
            .Set_environment_variable(
                Manager
                    .Get_current_task_identifier()
                    .expect("Failed to get current task identifier"),
                "Key",
                "Value",
            )
            .unwrap();
        let (_, _) = Manager
            .New_task(None, None, "Child Task", None, move || {
                let Current_task = Get_instance().unwrap().Get_current_task().unwrap();

                assert_eq!(
                    Current_task.Get_environment_variable("Key").unwrap(),
                    "Value"
                );
            })
            .unwrap();
    }

    fn Test_join_handle(Manager: &Manager_type) {
        let (_, Join_handle) = Manager
            .New_task(None, None, "Task with join handle", None, || 42)
            .unwrap();
        let Result = Join_handle.Join();
        assert_eq!(Result.unwrap(), 42);
    }
}
