// - Dependencies
use super::*;

// - Submodules
mod Lifecycle;
mod Metadata;
mod Properties;
mod Registration;
mod Relationships;
mod Signals;
mod Spawner;
mod Utilities;

#[cfg(test)]
mod Tests;

// - Re-exports

pub(crate) use Metadata::*;

// Manager module - core Manager structure and initialization

use crate::Manager::Metadata_type;

use alloc::collections::BTreeMap;
use Synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, rwlock::RwLock,
};

static Manager_instance: OnceLock<Manager_type> = OnceLock::new();

pub fn Initialize() -> &'static Manager_type {
    Manager_instance.get_or_init(Manager_type::New)
}

pub fn Get_instance() -> &'static Manager_type {
    Manager_instance.try_get().expect("Manager not initialized")
}

pub(crate) struct Inner_type {
    pub(crate) Tasks: BTreeMap<Task_identifier_type, Metadata_type>,
    pub(crate) Identifiers: BTreeMap<usize, Task_identifier_type>,
    pub(crate) Spawners: BTreeMap<usize, ::embassy_executor::Spawner>,
}

unsafe impl Send for Manager_type {}

/// A manager for tasks.
pub struct Manager_type(pub(crate) RwLock<CriticalSectionRawMutex, Inner_type>);

impl Manager_type {
    pub const Root_task_identifier: Task_identifier_type = Task_identifier_type::New(0);

    /// Create a new task manager instance,
    /// create a root task and register current thread as the root task main thread.
    pub(crate) fn New() -> Self {
        Manager_type(RwLock::new(Inner_type {
            Tasks: BTreeMap::new(),
            Identifiers: BTreeMap::new(),
            Spawners: BTreeMap::new(),
        }))
    }
}
