use core::mem::replace;
use core::time::Duration;

use xila::task::TaskIdentifier;

use crate::GlobalContext;

#[derive(Debug, Clone)]
pub enum EnvironmentState {
    Sleeping(Duration),
    Running,
    Exited,
}

pub struct EnvironmentContext {
    task: TaskIdentifier,
    state: EnvironmentState,
}

impl EnvironmentContext {
    pub fn new(task: TaskIdentifier) -> Self {
        Self {
            task,
            state: EnvironmentState::Running,
        }
    }

    pub unsafe fn get<'a>() -> Option<&'a mut Self> {
        unsafe { GlobalContext::get_environment_context() }
    }

    pub fn get_task(&self) -> TaskIdentifier {
        self.task
    }

    pub fn sleep(&mut self, duration: Duration) {
        self.state = EnvironmentState::Sleeping(duration);
    }

    pub fn suspend(&mut self) {
        self.state = EnvironmentState::Sleeping(Duration::MAX);
    }

    pub fn yield_now(&mut self) {
        self.state = EnvironmentState::Sleeping(Duration::ZERO);
    }

    pub fn wake_up(&mut self) {
        self.state = EnvironmentState::Running;
    }

    pub fn exit(&mut self) {
        self.state = EnvironmentState::Exited;
    }

    pub fn take_state(&mut self) -> EnvironmentState {
        replace(&mut self.state, EnvironmentState::Running)
    }
}
