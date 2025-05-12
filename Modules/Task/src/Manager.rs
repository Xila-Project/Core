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
use embassy_time::Timer;
use smol_str::SmolStr;
use Synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
    signal::Signal, waitqueue::WakerRegistration, Arc,
};

use core::{
    future::{poll_fn, Future},
    ptr::NonNull,
    task::Poll,
};

use core::time::Duration;
use Users::{Group_identifier_type, User_identifier_type};

/// Internal representation of a task.
struct Metadata_type {
    /// Internal identifier of the task.
    Internal_identifier: usize,
    /// Name of the task.
    Name: SmolStr,
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

pub fn Initialize() -> &'static Manager_type {
    return Manager_instance.get_or_init(Manager_type::New);
}

pub async fn Get_instance() -> &'static Manager_type {
    Manager_instance.get().await
}

struct Inner_type {
    Tasks: BTreeMap<Task_identifier_type, Metadata_type>,
    Identifiers: BTreeMap<usize, Task_identifier_type>,
    Spawners: Vec<SendSpawner>,
}

/// A manager for tasks.
pub struct Manager_type(RwLock<CriticalSectionRawMutex, Inner_type>);

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
    pub async fn Get_name(&self, Task_identifier: Task_identifier_type) -> Result_type<String> {
        Ok(self
            .0
            .read()
            .await
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Name
            .as_str()
            .to_string())
    }

    /// Spawn task
    pub async fn Spawn<Function_type, Future_type, Return_type>(
        &'static self,
        Parent_task: Task_identifier_type,
        Name: &str,
        Function: Function_type,
    ) -> Result_type<Join_handle_type<Return_type>>
    where
        Function_type: FnOnce(Task_identifier_type) -> Future_type + 'static + Send,
        Future_type: Future<Output = Return_type> + 'static + Send,
        Return_type: 'static + Send,
    {
        let Inner = self.0.read().await;

        // - Get parent task information if any (inheritance)
        let (Parent_task_identifier, Parent_environment_variables, User, Group) =
            if Inner.Tasks.len() > 0 {
                let Task = Inner
                    .Tasks
                    .get(&Parent_task)
                    .ok_or(Error_type::Invalid_task_identifier)?;

                (
                    Parent_task,
                    Task.Environment_variables.clone(),
                    Task.User,
                    Task.Group,
                )
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

        let (Join_handle_parent, Join_handle_child) = Join_handle_type::New();

        let Token = Pool.spawn(async move || {
            let Internal_identifier = Self::Get_current_internal_identifier().await;

            let Metadata = Metadata_type {
                Internal_identifier,
                Name,
                Parent: Parent_task_identifier,
                User,
                Group,
                Environment_variables: Parent_environment_variables,
                Signals: Signal_accumulator_type::New(),
            };

            let Child_task = Self::Get_new_task_identifier(&self.0.read().await.Tasks)
                .expect("Failed to get new task identifier");

            self.Register(Child_task, Metadata).await.unwrap();

            let Result = Function(Child_task).await;

            self.Unregister(Child_task)
                .await
                .expect("Failed to unregister task");

            Join_handle_child.Signal(Result);
        });

        self.0
            .read()
            .await
            .Spawners
            .first()
            .unwrap()
            .spawn(Token)
            .expect("Failed to spawn task");

        Ok(Join_handle_parent)
    }

    /// Register a task with its parent task.
    ///
    /// This function check if the task identifier is not already used,
    /// however it doesn't check if the parent task exists.
    async fn Register(
        &self,
        Task_identifier: Task_identifier_type,
        Task_internal: Metadata_type,
    ) -> Result_type<()> {
        if self
            .0
            .write()
            .await
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
    pub async fn Get_children(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Task_identifier_type>> {
        Ok(self
            .0
            .read()
            .await
            .Tasks
            .iter()
            .filter(|(_, Task)| Task.Parent == Task_identifier)
            .map(|(Identifier, _)| *Identifier)
            .collect())
    }

    /// Get the parent task of a task.
    pub async fn Get_parent(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Task_identifier_type> {
        Ok(self
            .0
            .read()
            .await
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Parent)
    }

    pub async fn Set_user(
        &self,
        Task_identifier: Task_identifier_type,
        User: User_identifier_type,
    ) -> Result_type<()> {
        self.0
            .write()
            .await
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .User = User;

        Ok(())
    }

    pub async fn Set_group(
        &self,
        Task_identifier: Task_identifier_type,
        Group: Group_identifier_type,
    ) -> Result_type<()> {
        self.0
            .write()
            .await
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Group = Group;

        Ok(())
    }

    /// Get user identifier of the owner of a task.
    pub async fn Get_user(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<User_identifier_type> {
        Ok(self
            .0
            .read()
            .await
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .User)
    }

    /// Get group identifier of the owner of a task.
    pub async fn Get_group(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Group_identifier_type> {
        Ok(self
            .0
            .read()
            .await
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Group)
    }

    /// Unregister task.
    ///
    /// If the task has children tasks, the root task adopts them.
    async fn Unregister(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Metadata_type> {
        // - Root task adopts all children of the task
        let mut Inner = self.0.write().await;

        Inner.Tasks.iter_mut().for_each(|(_, Task)| {
            if Task.Parent == Task_identifier {
                Task.Parent = Self::Root_task_identifier;
            }
        });

        // - Remove the task
        Inner
            .Tasks
            .remove(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)
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
            .await
            .Identifiers
            .get(&Internal_identifier)
            .expect("Failed to get task identifier")
    }

    pub async fn Get_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result_type<Environment_variable_type> {
        Ok(self
            .0
            .read()
            .await
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .iter()
            .find(|Variable| Variable.Get_name() == Name)
            .ok_or(Error_type::Invalid_environment_variable)?
            .clone())
    }

    pub async fn Get_environment_variables(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Environment_variable_type>> {
        Ok(self
            .0
            .read()
            .await
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .clone())
    }

    pub async fn Set_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
        Value: &str,
    ) -> Result_type<()> {
        let Environment_variable = Environment_variable_type::New(Name, Value);

        self.0
            .write()
            .await
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .push(Environment_variable);

        Ok(())
    }

    pub async fn Remove_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result_type<()> {
        self.0
            .write()
            .await
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Environment_variables
            .retain(|Variable| Variable.Get_name() != Name);

        Ok(())
    }

    pub async fn Pop_signal(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Option<Signal_type>> {
        Ok(self
            .0
            .write()
            .await
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)?
            .Signals
            .Pop())
    }

    pub async fn Peek_signal(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Option<Signal_type>> {
        Ok(self
            .0
            .write()
            .await
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

    #[tokio::test]
    async fn Test_task_manager() {
        let Manager = Initialize();

        // Run test sequentially since the instance is shared

        println!("Run test : Test_get_task_name");
        Test_get_task_name(Manager);
        println!("Run test : Test_Spawn");
        Test_spawn(Manager);
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

    async fn Test_get_task_name(Manager: &'static Manager_type) {
        let Task_name = "Test Task";
        let Task = Manager.Get_current_task_identifier().await;

        let _ = Manager
            .Spawn(Task, Task_name, async move |Task| {
                assert_eq!(
                    Get_instance().await.Get_name(Task).await.unwrap(),
                    Task_name
                );
            })
            .await
            .unwrap()
            .Join()
            .await;
    }

    async fn Test_spawn(Manager: &'static Manager_type) {
        let Task_name = "Child Task";
        let Task = Manager.Get_current_task_identifier().await;

        let _ = Manager.Spawn(Task, Task_name, async |_| {}).await.unwrap();
    }

    async fn Test_get_owner(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().await;

        assert_eq!(
            Get_instance().await.Get_user(Task).await.unwrap(),
            User_identifier_type::Root
        );
        assert_eq!(
            Get_instance().await.Get_group(Task).await.unwrap(),
            Group_identifier_type::Root
        );
    }

    async fn Test_get_current_task_identifier(Manager: &'static Manager_type) {
        let Task = Manager.Get_current_task_identifier().await;

        Manager
            .Spawn(Task, "Current Task", async move |Task| {
                assert_eq!(
                    Get_instance().await.Get_current_task_identifier().await,
                    Task
                );
            })
            .await
            .unwrap()
            .Join()
            .await;
    }

    async fn Test_task_owner_inheritance(Manager: &'static Manager_type) {
        let Task = Manager.Get_current_task_identifier().await;
        let User_identifier = User_identifier_type::New(123); // Assuming User_identifier_type is i32 for example
        let Group_identifier = Group_identifier_type::New(456); // Assuming Group_identifier_type is i32 for example

        Manager.Set_user(Task, User_identifier).await.unwrap();
        Manager.Set_group(Task, Group_identifier).await.unwrap();

        let _ = Manager
            .Spawn(Task, "Task 1", async move |Task_1| {
                assert_eq!(
                    Get_instance().await.Get_user(Task_1).await.unwrap(),
                    User_identifier
                );

                let _ = Get_instance()
                    .await
                    .Spawn(Task_1, "Task 2", async move |Task_2| {})
                    .await
                    .unwrap()
                    .Join()
                    .await;

                /*
                // - Inherit owner
                let _ = Get_instance()
                    .await
                    .Spawn(Task_1, "Task 2", async move |Task_2| {
                        assert_eq!(
                            Get_instance().await.Get_user(Task_2).await.unwrap(),
                            User_identifier
                        );
                        assert_eq!(
                            Get_instance().await.Get_group(Task_2).await.unwrap(),
                            Group_identifier
                        );

                        assert_eq!(Get_instance().await.Get_name(Task_2).await.unwrap(), "Task 2");

                        Manager_type::Sleep(std::time::Duration::from_secs(1)).await;
                    })
                    .await
                    .unwrap()
                    .Join()
                    .await;

                let User_identifier = User_identifier_type::New(6969); // Assuming User_identifier_type is i32 for example
                let Group_identifier = Group_identifier_type::New(4242); // Assuming Group_identifier_type is i32 for example

                // - Overwrite owner
                let _ = Get_instance()
                    .await
                    .Spawn(Task_1, "Task 3", async move |Task_3| {
                        Get_instance()
                            .await
                            .Set_user(Task_3, User_identifier)
                            .await
                            .unwrap();
                        Get_instance()
                            .await
                            .Set_group(Task_3, Group_identifier)
                            .await
                            .unwrap();

                        assert_eq!(
                            Get_instance().await.Get_user(Task_3).await.unwrap(),
                            User_identifier
                        );
                        assert_eq!(
                            Get_instance().await.Get_group(Task_3).await.unwrap(),
                            Group_identifier
                        );

                        assert_eq!(Get_instance().await.Get_name(Task_3).await.unwrap(), "Task 3");
                    })
                    .await
                    .unwrap()
                    .Join()
                    .await;
                */
            })
            .await
            .unwrap();
    }

    async fn Test_environment_variables(Manager: &Manager_type) {
        let Task_identifier = Manager.Get_current_task_identifier().await;
        let Name = "Key";
        let Value = "Value";

        Manager
            .Set_environment_variable(Task_identifier, Name, Value)
            .await
            .unwrap();
        assert_eq!(
            Manager
                .Get_environment_variable(Task_identifier, Name)
                .await
                .unwrap()
                .Get_value(),
            Value
        );
        Manager
            .Remove_environment_variable(Task_identifier, Name)
            .await
            .unwrap();
        assert!(Manager
            .Get_environment_variable(Task_identifier, Name)
            .await
            .is_err());
    }

    async fn Test_environment_variable_inheritance(Manager: &'static Manager_type) {
        let Task = Manager.Get_current_task_identifier().await;

        let _ = Manager
            .Spawn(Task, "Child Task", async move |Task| {
                Get_instance()
                    .await
                    .Set_environment_variable(Task, "Key", "Value")
                    .await
                    .unwrap();

                let _ = Get_instance()
                    .await
                    .Spawn(Task, "Grand child Task", async move |Task| {
                        assert_eq!(
                            Get_instance()
                                .await
                                .Get_environment_variable(Task, "Key")
                                .await
                                .unwrap()
                                .Get_value(),
                            "Value"
                        );
                    })
                    .await
                    .unwrap();
            })
            .await
            .unwrap();
    }

    async fn Test_join_handle(Manager: &'static Manager_type) {
        let Task = Manager.Get_current_task_identifier().await;

        let Join_handle = Manager
            .Spawn(Task, "Task with join handle", async |_| 42)
            .await
            .unwrap();

        assert_eq!(Join_handle.Join().await, 42);
    }

    async fn Test_set_user(Manager: &'static Manager_type) {
        let Task = Manager.Get_current_task_identifier().await;

        let User = User_identifier_type::New(123); // Assuming User_identifier_type is i32 for example

        Manager.Set_user(Task, User).await.unwrap();

        assert_eq!(Manager.Get_user(Task).await.unwrap(), User);
    }

    async fn Test_set_group(Manager: &Manager_type) {
        let Task = Manager.Get_current_task_identifier().await;

        let Group = Group_identifier_type::New(456); // Assuming Group_identifier_type is i32 for example

        Manager.Set_group(Task, Group).await.unwrap();

        assert_eq!(Manager.Get_group(Task).await.unwrap(), Group);
    }

    async fn Test_signal(Manager: &'static Manager_type) {
        let Task = Manager.Get_current_task_identifier().await;

        let _ = Manager
            .Spawn(Task, "Task with signal", async |_| {
                let Task = Get_instance().await.Get_current_task_identifier().await;

                Manager_type::Sleep(Duration::from_millis(10));

                assert_eq!(
                    Get_instance().await.Peek_signal(Task).await.unwrap(),
                    Some(Signal_type::Hangup)
                );

                assert_eq!(
                    Get_instance().await.Pop_signal(Task).await.unwrap(),
                    Some(Signal_type::Kill)
                );
            })
            .await
            .unwrap();

        Get_instance()
            .await
            .0
            .write()
            .await
            .Tasks
            .get_mut(&Task)
            .unwrap()
            .Signals
            .Send(Signal_type::Kill);

        Get_instance()
            .await
            .0
            .write()
            .await
            .Tasks
            .get_mut(&Task)
            .unwrap()
            .Signals
            .Send(Signal_type::Hangup);
    }
}
