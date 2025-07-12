use futures::block_on;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use task::TaskIdentifier;

pub static CONTEXT: Context = Context::new();

pub fn get_instance() -> &'static Context {
    &CONTEXT
}

struct Inner {
    task: Option<TaskIdentifier>,
}

pub struct Context(RwLock<CriticalSectionRawMutex, Inner>);

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub const fn new() -> Self {
        Self(RwLock::new(Inner { task: None }))
    }

    pub fn get_current_task_identifier(&self) -> TaskIdentifier {
        block_on(self.0.read()).task.expect("No current task set")
    }

    pub async fn set_task(&self, task: TaskIdentifier) {
        loop {
            let mut inner = self.0.write().await;

            if inner.task.is_none() {
                inner.task.replace(task);
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
        let task = task::get_instance().get_current_task_identifier().await;
        self.set_task(task).await;
        let result = function().await;
        self.clear_task().await;
        result
    }
}
