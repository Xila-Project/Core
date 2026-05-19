use core::time::Duration;

use xila::task::TaskIdentifier;

unsafe extern "Rust" {
    pub unsafe fn __wasm_get_environment_data() -> *mut EnvironmentContext;
}

#[derive(Debug, Clone)]
pub enum EnvironmentState {
    Sleep(Duration),
    Running,
    Exited,
}

pub struct EnvironmentContext {
    task: TaskIdentifier,
    state: EnvironmentState,
}

impl EnvironmentContext {
    pub unsafe fn get<'a>() -> &'a mut Self {
        unsafe { &mut *__wasm_get_environment_data() }
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

    pub fn wake_up(&mut self) {
        self.state = EnvironmentState::Running;
    }

    pub fn exit(&mut self) {
        self.state = EnvironmentState::Exited;
    }

    pub fn get_state(&self) -> &EnvironmentState {
        &self.state
    }
}
