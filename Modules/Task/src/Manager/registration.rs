// Registration module - handles task registration and basic task data

use super::*;
use crate::Manager::Metadata_type;
use alloc::vec::Vec;
use smol_str::SmolStr;
use users::{Group_identifier_type, User_identifier_type};

impl Manager_type {
    pub(crate) async fn Register(
        &self,
        parent: Task_identifier_type,
        name: &str,
    ) -> Result_type<Task_identifier_type> {
        let mut inner = self.0.write().await;

        // - Get parent task information if any (inheritance)
        let (Parent_task_identifier, Parent_environment_variables, User, Group) =
            if inner.tasks.is_empty() {
                (
                    Manager_type::ROOT_TASK_IDENTIFIER, // Root task is its own parent
                    Vec::new(),
                    User_identifier_type::ROOT,
                    Group_identifier_type::ROOT,
                )
            } else if let Ok(Metadata) = inner
                .tasks
                .get(&parent)
                .ok_or(Error_type::Invalid_task_identifier)
            {
                (
                    parent,
                    Metadata.Environment_variables.clone(),
                    Metadata.User,
                    Metadata.Group,
                )
            } else {
                (
                    Manager_type::ROOT_TASK_IDENTIFIER, // If parent task not found, use root task
                    Vec::new(),
                    User_identifier_type::ROOT,
                    Group_identifier_type::ROOT,
                )
            };

        // Truncate the name if it's too long
        let Name = if name.len() > 23 {
            // Truncate the name to 32 characters if it's too long
            &name[..23]
        } else {
            name
        };

        let Name = SmolStr::new_inline(Name);

        let Metadata = Metadata_type {
            Internal_identifier: 0, // Will be set later
            Name: Name.clone(),
            Parent: Parent_task_identifier,
            User,
            Group,
            Environment_variables: Parent_environment_variables,
            Signals: Signal_accumulator_type::new(),
            Spawner_identifier: 0, // Will be set later
        };

        let Identifier = Self::Find_first_available_identifier(
            &inner.tasks,
            (Task_identifier_inner_type::MIN..Task_identifier_inner_type::MAX).step_by(1),
        )
        .ok_or(Error_type::Too_many_tasks)?;

        // Find the first available task identifier
        //let Expected = Task_identifier_type::New(Identifier);

        // Populate the identifier with the first available one
        if inner
            .tasks
            .insert(
                Identifier, Metadata, // We insert None to reserve the identifier
            )
            .is_some()
        {
            unreachable!("Task identifier already exists");
        }

        Ok(Identifier)
    }

    /// Unregister task.
    ///
    /// If the task has children tasks, the root task adopts them.
    pub(crate) async fn Unregister(
        &self,
        identifier: Task_identifier_type,
    ) -> Result_type<Metadata_type> {
        let mut inner = self.0.write().await;

        // Root task adopts all children of the task
        inner.tasks.iter_mut().for_each(|(_, Task)| {
            if Task.Parent == identifier {
                Task.Parent = Self::ROOT_TASK_IDENTIFIER;
            }
        });

        let Metadata = inner
            .tasks
            .remove(&identifier)
            .ok_or(Error_type::Invalid_task_identifier)?;

        // Remove the internal identifier of the task
        inner.identifiers.remove(&Metadata.Internal_identifier);

        Ok(Metadata)
    }
}
