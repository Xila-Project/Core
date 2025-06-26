// Relationships module - handles parent/child relationships between tasks

use alloc::vec::Vec;

use super::*;

impl Manager_type {
    /// Get the children tasks of a task.
    pub async fn Get_children(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Task_identifier_type>> {
        Ok(self
            .0
            .read()
            .await
            .Tasks
            .iter()
            .filter(|(_, Metadata)| Metadata.Parent == Task_identifier)
            .map(|(Identifier, _)| *Identifier)
            .collect())
    }

    /// Get the parent task of a task.
    pub async fn Get_parent(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Task_identifier_type> {
        Self::Get_task(&*self.0.read().await, Task_identifier).map(|Task| Task.Parent)
    }
}
