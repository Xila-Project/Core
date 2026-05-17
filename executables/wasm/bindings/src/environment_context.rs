extern crate alloc;

use core::fmt::Debug;
use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::{collections::btree_map::BTreeMap, vec, vec::Vec};
use core::time::Duration;
use smol_str::SmolStr;
use xila::file_system::{Path, PathOwned};
use xila::task::TaskIdentifier;
use xila::virtual_file_system::{SynchronousDirectory, SynchronousFile};

use crate::host::bindings::common::identifier::FileSystemIdentifier;

unsafe extern "Rust" {
    fn get_global_context() -> NonNull<EnvironmentContext>;
}

pub struct EnvironmentContext {
    task: TaskIdentifier,
    state: EnvironmentState,
}

impl EnvironmentContext {
    pub const fn new(task: TaskIdentifier) -> Self {
        Self {
            task,
            state: EnvironmentState::Running,
        }
    }

    pub unsafe fn get_global<'a>() -> &'a mut Self {
        unsafe { get_global_context().as_mut() }.expect("Failed to get global context")
    }

    pub fn from_raw<'a>(environment: *mut EnvironmentContext) -> Option<&'a mut Self> {
        if environment.is_null()
            || !(environment as usize).is_multiple_of(core::mem::align_of::<Self>())
        {
            return None;
        }

        unsafe { Some(&mut *environment) }
    }

    pub fn take_from_environment<'a>(environment: *mut EnvironmentContext) -> Option<Box<Self>> {
        if environment.is_null()
            || !(environment as usize).is_multiple_of(core::mem::align_of::<Self>())
        {
            return None;
        }

        unsafe { Some(Box::from_raw(environment)) }
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

impl Debug for EnvironmentContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Context")
            .field("task", &self.task)
            .field("state", &self.state)
            .finish()
    }
}
