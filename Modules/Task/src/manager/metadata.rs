// Metadata module - handles task metadata and data structures

use super::*;
use alloc::vec::Vec;
use smol_str::SmolStr;
use users::{Group_identifier_type, User_identifier_type};

/// Internal representation of a task.
pub(crate) struct Metadata_type {
    /// Internal identifier of the task.
    pub(crate) internal_identifier: usize,
    /// Name of the task.
    pub(crate) name: SmolStr,
    /// The children of the task.
    pub(crate) parent: Task_identifier_type,
    /// The identifier of the user that owns the task.
    pub(crate) user: User_identifier_type,
    /// The identifier of the group that owns the task.
    pub(crate) group: Group_identifier_type,
    /// Environment variables of the task.
    pub(crate) environment_variables: Vec<Environment_variable_type>,
    /// Signals
    pub(crate) signals: Signal_accumulator_type,
    /// Index of the spawner that spawned this task (for tracking completion)
    pub(crate) spawner_identifier: usize,
}
