// Metadata module - handles task metadata and data structures

use super::*;
use smol_str::SmolStr;
use Users::{Group_identifier_type, User_identifier_type};

/// Internal representation of a task.
pub(crate) struct Metadata_type {
    /// Internal identifier of the task.
    pub(crate) Internal_identifier: usize,
    /// Name of the task.
    pub(crate) Name: SmolStr,
    /// The children of the task.
    pub(crate) Parent: Task_identifier_type,
    /// The identifier of the user that owns the task.
    pub(crate) User: User_identifier_type,
    /// The identifier of the group that owns the task.
    pub(crate) Group: Group_identifier_type,
    /// Environment variables of the task.
    pub(crate) Environment_variables: Vec<Environment_variable_type>,
    /// Signals
    pub(crate) Signals: Signal_accumulator_type,
    /// Index of the spawner that spawned this task (for tracking completion)
    pub(crate) Spawner_identifier: usize,
}
