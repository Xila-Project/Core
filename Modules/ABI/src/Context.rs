use Futures::block_on;
use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use Task::Task_identifier_type;

pub static Context: Context_type = Context_type::New();

pub fn Get_instance() -> &'static Context_type {
    &Context
}

struct Inner_type {
    Task: Option<Task_identifier_type>,
}

pub struct Context_type(RwLock<CriticalSectionRawMutex, Inner_type>);

impl Context_type {
    pub const fn New() -> Self {
        Self(RwLock::new(Inner_type { Task: None }))
    }

    pub fn Get_current_task_identifier(&self) -> Task_identifier_type {
        block_on(self.0.read()).Task.expect("No current task set")
    }

    pub async fn Set_task(&self, Task: Task_identifier_type) {
        loop {
            let mut Inner = self.0.write().await;

            if Inner.Task.is_none() {
                Inner.Task.replace(Task);
                break;
            }
        }
    }

    pub async fn Clear_task(&self) {
        let mut Inner = self.0.write().await;
        Inner.Task.take();
    }

    pub async fn Call_ABI<F, Fut, R>(&self, Function: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: core::future::Future<Output = R>,
    {
        let Task = Task::Get_instance().Get_current_task_identifier().await;
        self.Set_task(Task).await;
        let Result = Function().await;
        self.Clear_task().await;
        Result
    }
}
