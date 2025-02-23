// - Dependencies
// - - Local
use super::*;
// - - External
// - - - Standard library

extern crate alloc;

use alloc::collections::BTreeMap;
use embassy_executor::{
    raw::{task_from_waker, TaskPool},
    SendSpawner,
};
use embassy_futures::yield_now;
use embassy_sync::waitqueue::WakerRegistration;
use embassy_time::Timer;
use smol_str::SmolStr;

use core::{
    future::{poll_fn, Future},
    ptr::NonNull,
    task::{Poll, Waker},
};

use core::{slice, time::Duration};
use std::sync::{OnceLock, RwLock};
use Users::{Group_identifier_type, User_identifier_type};

/// Internal representation of a task.
struct Metadata_type {
    /// Internal identifier of the task.
    Internal_identifier: usize,
    /// Name of the task.
    Name: SmolStr,
    /// Result
    Waker: WakerRegistration,

    Result: Option<usize>,
    /// The children of the task.
    Parent: Task_identifier_type,
    /// The identifier of the user that owns the task.
    User: User_identifier_type,
    /// The identifier of the group that owns the task.
    Group: Group_identifier_type,
    /// Environment variables of the task.
    Environment_variables: Vec<Environment_variable_type>,
    /// Signals
    Signals: Signal_accumulator_type,
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

struct Inner_type {
    Tasks: BTreeMap<Task_identifier_type, Metadata_type>,
    Identifiers: BTreeMap<usize, Task_identifier_type>,
    Spawners: Vec<SendSpawner>,
}

/// A manager for tasks.
pub struct Manager_type(RwLock<Inner_type>);

impl Manager_type {
    pub const Root_task_identifier: Task_identifier_type = Task_identifier_type::New(0);

    /// Create a new task manager instance,
    /// create a root task and register current thread as the root task main thread.
    fn New() -> Self {
        let Manager = Manager_type(RwLock::new(Inner_type {
            Tasks: BTreeMap::new(),
            Identifiers: BTreeMap::new(),
            Spawners: Vec::new(),
        }));

        Manager
    }

    fn Get_new_task_identifier(
        Tasks: &BTreeMap<Task_identifier_type, Metadata_type>,
    ) -> Result_type<Task_identifier_type> {
        (0..Task_identifier_type::Maximum)
            .map(Task_identifier_type::from)
            .find(|Identifier| !Tasks.contains_key(Identifier))
            .ok_or(Error_type::Too_many_tasks)
    }

    /// # Arguments
    /// * `Task_identifier` - The identifier of the task.
    pub fn Get_name(&self, Task_identifier: Task_identifier_type) -> Result_type<String> {
        Ok(self
            .0
            .read()?
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Name
            .as_str()
            .to_string())
    }

    pub async fn Join(&self, Task_identifier: Task_identifier_type) -> Result_type<usize> {
        let mut Task = self
            .0
            .write()?
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?;

        if Task.Waker.occupied() {
            return Err(Error_type::Already_set);
        }

        poll_fn(|Context| {
            Task.Waker.register(Context.waker());
        })
        .await
        .map_err(|_| Error_type::Poisoned_lock)
    }

    /// Spawn task
    async fn Spawn<Function_type, Future_type>(
        &'static self,
        Parent_task: Task_identifier_type,
        Function: Function_type,
        Name: &str,
    ) -> Result_type<()>
    where
        Function_type: FnOnce(Task_identifier_type) -> Future_type + 'static + Send,
        Future_type: Future<Output = usize> + 'static + Send,
    {
        let Inner = self.0.read()?;

        // - Get parent task information if any (inheritance)
        let (Parent_task_identifier, Parent_environment_variables, User, Group) =
            if Inner.Tasks.len() > 0 {
                let Parent_environment_variables = Inner
                    .Tasks
                    .get(&Parent_task)
                    .ok_or(Error_type::Invalid_task_identifier)?
                    .Environment_variables
                    .clone();

                let User = Inner
                    .Tasks
                    .get(&Parent_task)
                    .ok_or(Error_type::Invalid_task_identifier)?
                    .User;

                let Group = Inner
                    .Tasks
                    .get(&Parent_task)
                    .ok_or(Error_type::Invalid_task_identifier)?
                    .Group;

                (Parent_task, Parent_environment_variables, User, Group)
            } else {
                (
                    Self::Root_task_identifier, // Root task is its own parent
                    Vec::new(),
                    User_identifier_type::Root,
                    Group_identifier_type::Root,
                )
            };

        drop(Inner); // Unlock the read lock

        let Pool = Box::new(TaskPool::<_, 1>::new());

        let Pool = Box::leak(Pool);

        let Name = SmolStr::new_inline(Name);

        let Token = Pool.spawn(async move || {
            let Internal_identifier = Self::Get_current_internal_identifier().await;

            let Metadata = Metadata_type {
                Internal_identifier,
                Name,
                Waker: WakerRegistration::new(),
                Result: None,
                Parent: Parent_task_identifier,
                User,
                Group,
                Environment_variables: Parent_environment_variables,
                Signals: Signal_accumulator_type::New(),
            };

            let Child_taks = Self::Get_new_task_identifier(
                &self
                    .0
                    .read()
                    .expect("Failed to get new task identifier")
                    .Tasks,
            )
            .expect("Failed to get new task identifier");

            self.Register(Child_taks, Metadata)
                .expect("Failed to register task");

            let Future = Function(Child_taks).await;

            self.Unregister(Child_taks)
                .await
                .expect("Failed to unregister task");
        });

        self.0
            .read()?
            .Spawners
            .first()
            .unwrap()
            .spawn(Token)
            .expect("Failed to spawn task");

        Ok(())
    }

    /// Register a task with its parent task.
    ///
    /// This function check if the task identifier is not already used,
    /// however it doesn't check if the parent task exists.
    fn Register(
        &self,
        Task_identifier: Task_identifier_type,
        Task_internal: Metadata_type,
    ) -> Result_type<()> {
        if self
            .0
            .write()?
            .Tasks
            .insert(Task_identifier, Task_internal)
            .is_some()
        {
            return Err(Error_type::Invalid_task_identifier);
        }

        Ok(())
    }

    pub async fn Yield() {
        yield_now().await;
    }

    /// Sleep the current thread for a given duration.
    pub async fn Sleep(Duration: Duration) {
        let Nano_seconds = Duration.as_nanos();

        Timer::after(embassy_time::Duration::from_nanos(Nano_seconds as u64)).await
    }

    /// Get the children tasks of a task.
    pub fn Get_children(
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
    pub fn Get_parent(
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

    pub fn Set_user(
        &self,
        Task_identifier: Task_identifier_type,
        User: User_identifier_type,
    ) -> Result_type<()> {
        self.0
            .write()?
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .User = User;

        Ok(())
    }

    pub fn Set_group(
        &self,
        Task_identifier: Task_identifier_type,
        Group: Group_identifier_type,
    ) -> Result_type<()> {
        self.0
            .write()?
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Group = Group;

        Ok(())
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
            .User)
    }

    /// Get group identifier of the owner of a task.
    pub fn Get_group(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Group_identifier_type> {
        Ok(self
            .0
            .read()?
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Group)
    }

    /// Unregister task.
    ///
    /// If the task has children tasks, the root task adopts them.
    async fn Unregister(&self, Task_identifier: Task_identifier_type) -> Result_type<()> {
        // - Root task adopts all children of the task
        let mut Inner = self.0.write()?;

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

    async fn Get_current_internal_identifier() -> usize {
        poll_fn(|Context| {
            let Task_reference = task_from_waker(Context.waker());

            let Inner: NonNull<u8> = unsafe { core::mem::transmute(Task_reference) };

            let Identifier = Inner.as_ptr() as usize;

            Poll::Ready(Identifier)
        })
        .await
    }

    pub async fn Get_current_task_identifier(&self) -> Task_identifier_type {
        let Internal_identifier = Self::Get_current_internal_identifier().await;

        *self
            .0
            .read()
            .expect("Failed to get task identifier")
            .Identifiers
            .get(&Internal_identifier)
            .expect("Failed to get task identifier")
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

    pub fn Pop_signal(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Option<Signal_type>> {
        Ok(self
            .0
            .write()?
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Signals
            .Pop())
    }

    pub fn Peek_signal(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Option<Signal_type>> {
        Ok(self
            .0
            .write()?
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Signals
            .Peek())
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
        println!("Run test : Test_Spawn");
        Test_Spawn(Manager);
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
        println!("Run test : Test_set_user");
        Test_set_user(Manager);
        println!("Run test : Test_set_group");
        Test_set_group(Manager);
        println!("Run test : Test_signal");
        Test_signal(Manager);
    }

    fn Test_get_task_name(Manager: &Manager_type) {
        let Task_name = "Test Task";
        let Task = Manager.Get_current_task_identifier().await.unwrap();

        let _ = Manager
            .Spawn(Task, Task_name, move || {
                let Task = Get_instance().Get_current_task_identifier().unwrap();

                assert_eq!(Get_instance().Get_task_name(Task).unwrap(), Task_name);
            })
            .unwrap();
    }

    fn Test_Spawn(Manager: &Manager_type) {
        let Task_name = "Child Task";
        let Task = Manager.Get_current_task_identifier().unwrap();

        let _ = Manager.Spawn(Task, Task_name, || {}).unwrap();
    }

    fn Test_get_owner(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();

        assert_eq!(
            Get_instance().Get_user(Task).unwrap(),
            User_identifier_type::Root
        );
        assert_eq!(
            Get_instance().Get_group(Task).unwrap(),
            Group_identifier_type::Root
        );
    }

    fn Test_get_current_task_identifier(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();

        let (_, Join_handle) = Manager
            .Spawn(Task, "Current Task", move || {
                let _ = Get_instance().Get_current_task_identifier().unwrap();
            })
            .unwrap();

        Join_handle.Join().unwrap();
    }

    fn Test_task_owner_inheritance(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();
        let User_identifier = User_identifier_type::New(123); // Assuming User_identifier_type is i32 for example
        let Group_identifier = Group_identifier_type::New(456); // Assuming Group_identifier_type is i32 for example

        Manager.Set_user(Task, User_identifier).unwrap();
        Manager.Set_group(Task, Group_identifier).unwrap();

        let _ = Manager
            .Spawn(Task, "Task 1", move || {
                let Task = Get_instance().Get_current_task_identifier().unwrap();

                assert_eq!(Get_instance().Get_user(Task).unwrap(), User_identifier);

                // - Inherit owner
                let _ = Get_instance()
                    .Spawn(Task, "Task 2", move || {
                        let Task = Get_instance().Get_current_task_identifier().unwrap();

                        assert_eq!(Get_instance().Get_user(Task).unwrap(), User_identifier);
                        assert_eq!(Get_instance().Get_group(Task).unwrap(), Group_identifier);

                        assert_eq!(Get_instance().Get_task_name(Task).unwrap(), "Task 2");

                        Manager_type::Sleep(std::time::Duration::from_secs(1));
                    })
                    .unwrap();

                let User_identifier = User_identifier_type::New(6969); // Assuming User_identifier_type is i32 for example
                let Group_identifier = Group_identifier_type::New(4242); // Assuming Group_identifier_type is i32 for example

                // - Overwrite owner
                let _ = Get_instance()
                    .Spawn(Task, "Task 3", move || {
                        let Task = Get_instance().Get_current_task_identifier().unwrap();

                        Get_instance().Set_user(Task, User_identifier).unwrap();
                        Get_instance().Set_group(Task, Group_identifier).unwrap();

                        assert_eq!(Get_instance().Get_user(Task).unwrap(), User_identifier);
                        assert_eq!(Get_instance().Get_group(Task).unwrap(), Group_identifier);

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
            .Spawn(Task, "Child Task", move || {
                let Current_task = Get_instance().Get_current_task_identifier().unwrap();

                Get_instance()
                    .Set_environment_variable(Task, "Key", "Value")
                    .unwrap();

                let _ = Get_instance()
                    .Spawn(Current_task, "Grandchild Task", || {
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

        let (_, Join_handle) = Manager.Spawn(Task, "Task with join handle", || 42).unwrap();
        let Result = Join_handle.Join();
        assert_eq!(Result.unwrap(), 42);
    }

    fn Test_set_user(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();

        let User = User_identifier_type::New(123); // Assuming User_identifier_type is i32 for example

        Manager.Set_user(Task, User).unwrap();

        assert_eq!(Manager.Get_user(Task).unwrap(), User);
    }

    fn Test_set_group(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();

        let Group = Group_identifier_type::New(456); // Assuming Group_identifier_type is i32 for example

        Manager.Set_group(Task, Group).unwrap();

        assert_eq!(Manager.Get_group(Task).unwrap(), Group);
    }

    fn Test_signal(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().unwrap();

        let _ = Manager
            .Spawn(Task, "Task with signal", || {
                let Task = Get_instance().Get_current_task_identifier().unwrap();

                Manager_type::Sleep(Duration::from_millis(10));

                assert_eq!(
                    Get_instance().Peek_signal(Task).unwrap(),
                    Some(Signal_type::Hangup)
                );

                assert_eq!(
                    Get_instance().Pop_signal(Task).unwrap(),
                    Some(Signal_type::Kill)
                );
            })
            .unwrap();

        Get_instance()
            .Tasks
            .write()
            .unwrap()
            .Tasks
            .get_mut(&Task)
            .unwrap()
            .Signals
            .Send(Signal_type::Kill);

        Get_instance()
            .Tasks
            .write()
            .unwrap()
            .Tasks
            .get_mut(&Task)
            .unwrap()
            .Signals
            .Send(Signal_type::Hangup);
    }
}
