// Relationships module - handles parent/child relationships between tasks

use alloc::vec::Vec;

use super::*;

impl Manager_type {
    /// Get the children tasks of a task.
    pub async fn get_children(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Task_identifier_type>> {
        Ok(self
            .0
            .read()
            .await
            .tasks
            .iter()
            .filter(|(_, Metadata)| Metadata.Parent == task_identifier)
            .map(|(identifier, _)| *identifier)
            .collect())
    }

    /// Get the parent task of a task.
    pub async fn get_parent(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Task_identifier_type> {
        Self::get_task(&*self.0.read().await, task_identifier).map(|Task| Task.Parent)
    }
}
