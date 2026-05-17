use core::ffi::c_void;

use xila::{shared::BijectiveBTreeMap, task::TaskIdentifier};

use crate::host::virtual_machine::WasmPointer;

#[derive(Debug, Clone)]
pub enum EnvironmentState {
    Sleep(Duration),
    Running,
}

#[derive(Debug, Clone, Default)]
pub struct CustomData {
    pub translation_map: BijectiveBTreeMap<WasmPointer, *mut c_void>,
    pub task: TaskIdentifier,
    pub state: EnvironmentState,
}

impl CustomData {
    pub fn new(task: TaskIdentifier) -> Self {
        Self {
            translation_map: BijectiveBTreeMap::new(),
            task,
            state: EnvironmentState::Running,
        }
    }

    pub fn get_current_task_identifier(&self) -> TaskIdentifier {
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

    pub fn get_state(&self) -> &EnvironmentState {
        &self.state
    }
}
