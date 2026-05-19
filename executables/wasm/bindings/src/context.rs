use core::{ffi::c_void, ptr::NonNull, time::Duration};

use alloc::sync::Arc;
use xila::{
    shared::BijectiveBTreeMap,
    synchronization::{
        mutex::Mutex, once_lock::OnceLock, raw::CriticalSectionRawMutex, rwlock::RwLock,
    },
    task::{TaskIdentifier, block_on, yield_now},
};

use crate::host::WasmPointer;

static GLOBAL_CONTEXT: Mutex<CriticalSectionRawMutex, Option<Context>> = Mutex::new(None);

#[derive(Debug)]
pub enum EnvironmentState {
    Sleep(Duration),
    Running,
    Exited,
}

#[derive(Debug)]
pub struct Context {
    pub task: TaskIdentifier,
    pub state: EnvironmentState,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    pub async unsafe fn set_global(context: Context) {
        loop {
            {
                let mut global_context = GLOBAL_CONTEXT.lock().await;
                if global_context.is_none() {
                    *global_context = Some(context);
                    break;
                }
            }

            yield_now().await;
        }
    }

    pub fn get_global<'a>() -> Option<&'a mut Context> {
        let guard = block_on(GLOBAL_CONTEXT.lock())?;
        guard.as_mut()
    }

    pub fn new(task: TaskIdentifier) -> Self {
        Self {
            task,
            state: EnvironmentState::Running,
        }
    }

    pub fn get_task(&self) -> TaskIdentifier {
        self.task
    }

    pub fn sleep(&mut self, duration: Duration) {
        self.state = EnvironmentState::Sleep(duration);
    }

    pub fn suspend(&mut self) {
        self.state = EnvironmentState::Sleep(Duration::MAX);
    }

    pub fn yield_now(&mut self) {
        self.state = EnvironmentState::Sleep(Duration::ZERO);
    }

    pub fn exit(&mut self) {
        self.state = EnvironmentState::Exited;
    }

    pub fn wake_up(&mut self) {
        self.state = EnvironmentState::Running;
    }

    pub fn get_state(&self) -> &EnvironmentState {
        &self.state
    }
}
