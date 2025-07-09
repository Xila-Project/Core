use futures::block_on;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use task::Task_identifier_type;

pub static CONTEXT: Context_type = Context_type::new();

pub fn get_instance() -> &'static Context_type {
    &CONTEXT
}

struct Inner_type {
    task: Option<Task_identifier_type>,
}

pub struct Context_type(RwLock<CriticalSectionRawMutex, Inner_type>);

impl Context_type {
    pub const fn new() -> Self {
        Self(RwLock::new(Inner_type { task: None }))
    }

    pub fn get_current_task_identifier(&self) -> Task_identifier_type {
        block_on(self.0.read()).task.expect("No current task set")
    }

    pub async fn set_task(&self, Task: Task_identifier_type) {
        loop {
            let mut Inner = self.0.write().await;

            if Inner.task.is_none() {
                Inner.task.replace(Task);
                break;
            }
        }
    }

    pub async fn clear_task(&self) {
        let mut inner = self.0.write().await;
        inner.task.take();
    }

    pub async fn call_abi<F, Fut, R>(&self, function: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: core::future::Future<Output = R>,
    {
        let Task = task::get_instance().get_current_task_identifier().await;
        self.set_task(Task).await;
        let result = function().await;
        self.clear_task().await;
        result
    }
}
