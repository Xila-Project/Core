// Registration module - handles task registration and basic task data

use super::*;
use crate::Manager::Metadata_type;
use smol_str::SmolStr;
use Users::{Group_identifier_type, User_identifier_type};

impl Manager_type {
    pub(crate) async fn Register(
        &self,
        Parent: Task_identifier_type,
        Name: &str,
    ) -> Result_type<Task_identifier_type> {
        let mut Inner = self.0.write().await;

        // - Get parent task information if any (inheritance)
        let (Parent_task_identifier, Parent_environment_variables, User, Group) =
            if Inner.Tasks.is_empty() {
                (
                    Manager_type::Root_task_identifier, // Root task is its own parent
                    Vec::new(),
                    User_identifier_type::Root,
                    Group_identifier_type::Root,
                )
            } else if let Ok(Metadata) = Inner
                .Tasks
                .get(&Parent)
                .ok_or(Error_type::Invalid_task_identifier)
            {
                (
                    Parent,
                    Metadata.Environment_variables.clone(),
                    Metadata.User,
                    Metadata.Group,
                )
            } else {
                (
                    Manager_type::Root_task_identifier, // If parent task not found, use root task
                    Vec::new(),
                    User_identifier_type::Root,
                    Group_identifier_type::Root,
                )
            };

        // Truncate the name if it's too long
        let Name = if Name.len() > 23 {
            // Truncate the name to 32 characters if it's too long
            &Name[..23]
        } else {
            Name
        };

        let Name = SmolStr::new_inline(Name);

        let Metadata = Metadata_type {
            Internal_identifier: 0, // Will be set later
            Name: Name.clone(),
            Parent: Parent_task_identifier,
            User,
            Group,
            Environment_variables: Parent_environment_variables,
            Signals: Signal_accumulator_type::New(),
            Spawner_identifier: 0, // Will be set later
        };

        let Identifier = Self::Find_first_available_identifier(
            &Inner.Tasks,
            (Task_identifier_inner_type::MIN..Task_identifier_inner_type::MAX).step_by(1),
        )
        .ok_or(Error_type::Too_many_tasks)?;

        // Find the first available task identifier
        //let Expected = Task_identifier_type::New(Identifier);

        // Populate the identifier with the first available one
        if Inner
            .Tasks
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
        Identifier: Task_identifier_type,
    ) -> Result_type<Metadata_type> {
        let mut Inner = self.0.write().await;

        // Root task adopts all children of the task
        Inner.Tasks.iter_mut().for_each(|(_, Task)| {
            if Task.Parent == Identifier {
                Task.Parent = Self::Root_task_identifier;
            }
        });

        let Metadata = Inner
            .Tasks
            .remove(&Identifier)
            .ok_or(Error_type::Invalid_task_identifier)?;

        // Remove the internal identifier of the task
        Inner.Identifiers.remove(&Metadata.Internal_identifier);

        Ok(Metadata)
    }
}
