// - Dependencies
use super::*;

// - Submodules
mod lifecycle;
mod metadata;
mod properties;
mod registration;
mod relationships;
mod signals;
mod spawner;
mod utilities;

#[cfg(test)]
mod tests;

// - Re-exports

pub(crate) use metadata::*;

// Manager module - core Manager structure and initialization

use crate::manager::Metadata;

use alloc::collections::BTreeMap;
use synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

static MANAGER_INSTANCE: OnceLock<Manager> = OnceLock::new();

pub fn initialize() -> &'static Manager {
    MANAGER_INSTANCE.get_or_init(Manager::new)
}

pub fn get_instance() -> &'static Manager {
    MANAGER_INSTANCE.try_get().expect("Manager not initialized")
}

pub(crate) struct Inner {
    pub(crate) tasks: BTreeMap<TaskIdentifier, Metadata>,
    pub(crate) identifiers: BTreeMap<usize, TaskIdentifier>,
    pub(crate) spawners: BTreeMap<usize, ::embassy_executor::Spawner>,
}

unsafe impl Send for Manager {}

/// A manager for tasks.
pub struct Manager(pub(crate) RwLock<CriticalSectionRawMutex, Inner>);

impl Manager {
    pub const ROOT_TASK_IDENTIFIER: TaskIdentifier = TaskIdentifier::new(0);

    /// Create a new task manager instance,
    /// create a root task and register current thread as the root task main thread.
    pub(crate) fn new() -> Self {
        Manager(RwLock::new(Inner {
            tasks: BTreeMap::new(),
            identifiers: BTreeMap::new(),
            spawners: BTreeMap::new(),
        }))
    }
}
