// Properties module - handles task properties like user, group, and environment variables

use alloc::string::ToString;

use super::*;
use alloc::{string::String, vec::Vec};
use Users::{Group_identifier_type, User_identifier_type};

impl Manager_type {
    pub async fn set_user(
        &self,
        task_identifier: Task_identifier_type,
        user: User_identifier_type,
    ) -> Result_type<()> {
        Self::Get_task_mutable(&mut *self.0.write().await, task_identifier)?.User = user;

        Ok(())
    }

    pub async fn Set_group(
        &self,
        task_identifier: Task_identifier_type,
        group: Group_identifier_type,
    ) -> Result_type<()> {
        Self::Get_task_mutable(&mut *self.0.write().await, task_identifier)?.Group = group;

        Ok(())
    }

    pub async fn Set_environment_variable(
        &self,
        task_identifier: Task_identifier_type,
        name: &str,
        value: &str,
    ) -> Result_type<()> {
        let environment_variable = Environment_variable_type::New(name, value);

        // Keep the write lock for the entire operation
        let mut Inner = self.0.write().await;
        let metadata = Self::Get_task_mutable(&mut Inner, task_identifier)?;

        // We remove any existing environment variable with the same name
        metadata
            .Environment_variables
            .retain(|variable| variable.Get_name() != name);
        // Add the new environment variable
        metadata.Environment_variables.push(environment_variable);

        Ok(())
    }

    pub async fn Set_environment_variables(
        &self,
        task_identifier: Task_identifier_type,
        environment_variables: &[(&str, &str)],
    ) -> Result_type<()> {
        let mut inner = self.0.write().await;
        let metadata = Self::Get_task_mutable(&mut inner, task_identifier)?;

        environment_variables.iter().for_each(|(Name, Value)| {
            let environment_variable = Environment_variable_type::New(Name, Value);
            metadata.Environment_variables.push(environment_variable);
        });

        Ok(())
    }

    pub async fn Remove_environment_variable(
        &self,
        task_identifier: Task_identifier_type,
        name: &str,
    ) -> Result_type<()> {
        Self::Get_task_mutable(&mut *self.0.write().await, task_identifier)?
            .Environment_variables
            .retain(|variable| variable.Get_name() != name);

        Ok(())
    }

    /// Get user identifier of the owner of a task.
    pub async fn Get_user(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<User_identifier_type> {
        Self::Get_task(&*self.0.read().await, task_identifier).map(|Task| Task.User)
    }

    /// Get group identifier of the owner of a task.
    pub async fn Get_group(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Group_identifier_type> {
        Self::Get_task(&*self.0.read().await, task_identifier).map(|Task| Task.Group)
    }

    pub async fn Get_environment_variable(
        &self,
        task_identifier: Task_identifier_type,
        name: &str,
    ) -> Result_type<Environment_variable_type> {
        Self::Get_task(&*self.0.read().await, task_identifier)?
            .Environment_variables
            .iter()
            .find(|variable| variable.Get_name() == name)
            .cloned()
            // If the variable is not found, return an error
            .ok_or(Error_type::Invalid_environment_variable)
    }

    pub async fn Get_environment_variables(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Environment_variable_type>> {
        Self::Get_task(&*self.0.read().await, task_identifier)
            .map(|task| task.Environment_variables.clone())
            .map_err(|_| Error_type::Invalid_task_identifier)
    }

    /// # Arguments
    /// * `Task_identifier` - The identifier of the task.
    pub async fn Get_name(&self, Task_identifier: Task_identifier_type) -> Result_type<String> {
        Self::Get_task(&*self.0.read().await, Task_identifier).map(|task| task.Name.to_string())
    }
}
