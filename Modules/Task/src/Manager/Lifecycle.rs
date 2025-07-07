// Lifecycle module - handles task spawning, execution, and lifecycle management

use super::*;
use alloc::boxed::Box;
use core::{
    future::{poll_fn, Future},
    ptr::NonNull,
    task::Poll,
    time::Duration,
};
use embassy_executor::raw::{task_from_waker, TaskPool};
use embassy_futures::yield_now;
use embassy_time::Timer;

impl Manager_type {
    // Static function to create and execute tasks
    // This function is outside the closure that captures SpawnToken,
    // so it can be called safely from nested tasks
    async fn Create_and_run_task<R: 'static, Function_type, Future_type>(
        manager: &'static Manager_type,
        parent_task_identifier: Task_identifier_type,
        name: &str,
        function: Function_type,
        spawner: Option<usize>,
    ) -> Result_type<(Join_handle_type<R>, Task_identifier_type)>
    where
        Function_type: FnOnce(Task_identifier_type) -> Future_type + 'static,
        Future_type: Future<Output = R> + 'static,
    {
        let Identifier = manager
            .Register(parent_task_identifier, name)
            .await
            .expect("Failed to get new task identifier");

        let Pool = Box::new(TaskPool::<_, 1>::new());
        let pool = Box::leak(Pool);

        let (Join_handle_parent, Join_handle_child) = Join_handle_type::new();

        let Task = async move || {
            let manager = Get_instance();

            let Internal_identifier = Manager_type::Get_current_internal_identifier().await;

            manager
                .Set_internal_identifier(Identifier, Internal_identifier)
                .await
                .expect("Failed to register task");

            let Result = function(Identifier).await;

            Join_handle_child.Signal(Result);

            manager
                .Unregister(Identifier)
                .await
                .expect("Failed to unregister task");
        };

        let mut Inner = manager.0.write().await;

        // Select the best spawner for the new task
        let Spawner = if let Some(Spawner) = spawner {
            if !Inner.spawners.contains_key(&Spawner) {
                return Err(Error_type::Invalid_spawner_identifier);
            }
            Spawner
        } else {
            Manager_type::Select_best_spawner(&Inner)?
        };

        Inner
            .tasks
            .get_mut(&Identifier)
            .expect("Failed to get task metadata")
            .Spawner_identifier = Spawner;

        let Token = pool.spawn(Task);

        Inner
            .spawners
            .get(&Spawner)
            .expect("Failed to get spawner")
            .spawn(Token)
            .expect("Failed to spawn task");

        Ok((Join_handle_parent, Identifier))
    }

    /// Spawn task
    pub async fn Spawn<Function_type, Future_type, Return_type>(
        &'static self,
        parent_task: Task_identifier_type,
        name: &str,
        spawner: Option<usize>,
        function: Function_type,
    ) -> Result_type<(Join_handle_type<Return_type>, Task_identifier_type)>
    where
        Function_type: FnOnce(Task_identifier_type) -> Future_type + 'static,
        Future_type: Future<Output = Return_type> + 'static,
        Return_type: 'static,
    {
        // Call the helper function with all our parameters
        Self::Create_and_run_task(self, parent_task, name, function, spawner).await
    }

    /// Set the internal identifier of a task.
    ///
    /// This function check if the task identifier is not already used,
    /// however it doesn't check if the parent task exists.
    async fn Set_internal_identifier(
        &self,
        identifier: Task_identifier_type,
        internal_identifier: usize,
    ) -> Result_type<()> {
        let mut inner = self.0.write().await;

        let Metadata = Self::Get_task_mutable(&mut inner, identifier)?;

        Metadata.Internal_identifier = internal_identifier;

        // Register the internal identifier of the task
        if let Some(Old_identifier) = inner.identifiers.insert(internal_identifier, identifier) {
            // Rollback the task registration if internal identifier registration fails
            inner.identifiers.remove(&internal_identifier);
            inner
                .identifiers
                .insert(internal_identifier, Old_identifier);
            return Err(Error_type::Invalid_task_identifier);
        }

        Ok(())
    }

    pub async fn Yield() {
        yield_now().await;
    }

    /// Sleep the current thread for a given duration.
    pub async fn Sleep(Duration: Duration) {
        let nano_seconds = Duration.as_nanos();

        Timer::after(embassy_time::Duration::from_nanos(nano_seconds as u64)).await
    }

    pub async fn Get_current_internal_identifier() -> usize {
        poll_fn(|context| {
            let task_reference = task_from_waker(context.waker());

            let Inner: NonNull<u8> = unsafe { core::mem::transmute(task_reference) };

            let Identifier = Inner.as_ptr() as usize;

            Poll::Ready(Identifier)
        })
        .await
    }

    pub async fn Get_current_task_identifier(&self) -> Task_identifier_type {
        let internal_identifier = Self::Get_current_internal_identifier().await;

        *self
            .0
            .read()
            .await
            .identifiers
            .get(&internal_identifier)
            .expect("Failed to get task identifier")
    }
}
