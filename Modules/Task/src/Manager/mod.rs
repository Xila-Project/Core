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

use crate::Manager::Metadata_type;

use alloc::collections::BTreeMap;
use synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

static MANAGER_INSTANCE: OnceLock<Manager_type> = OnceLock::new();

pub fn Initialize() -> &'static Manager_type {
    MANAGER_INSTANCE.get_or_init(Manager_type::New)
}

pub fn get_instance() -> &'static Manager_type {
    MANAGER_INSTANCE.try_get().expect("Manager not initialized")
}

pub(crate) struct Inner_type {
    pub(crate) tasks: BTreeMap<Task_identifier_type, Metadata_type>,
    pub(crate) identifiers: BTreeMap<usize, Task_identifier_type>,
    pub(crate) spawners: BTreeMap<usize, ::embassy_executor::Spawner>,
}

unsafe impl Send for Manager_type {}

/// A manager for tasks.
pub struct Manager_type(pub(crate) RwLock<CriticalSectionRawMutex, Inner_type>);

impl Manager_type {
    pub const ROOT_TASK_IDENTIFIER: Task_identifier_type = Task_identifier_type::new(0);

    /// Create a new task manager instance,
    /// create a root task and register current thread as the root task main thread.
    pub(crate) fn New() -> Self {
        Manager_type(RwLock::new(Inner_type {
            tasks: BTreeMap::new(),
            identifiers: BTreeMap::new(),
            spawners: BTreeMap::new(),
        }))
    }
}
