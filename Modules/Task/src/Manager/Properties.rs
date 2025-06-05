// Properties module - handles task properties like user, group, and environment variables

use alloc::string::ToString;

use super::*;
use alloc::{string::String, vec::Vec};
use Users::{Group_identifier_type, User_identifier_type};

impl Manager_type {
    pub async fn Set_user(
        &self,
        Task_identifier: Task_identifier_type,
        User: User_identifier_type,
    ) -> Result_type<()> {
        Self::Get_task_mutable(&mut *self.0.write().await, Task_identifier)?.User = User;

        Ok(())
    }

    pub async fn Set_group(
        &self,
        Task_identifier: Task_identifier_type,
        Group: Group_identifier_type,
    ) -> Result_type<()> {
        Self::Get_task_mutable(&mut *self.0.write().await, Task_identifier)?.Group = Group;

        Ok(())
    }

    pub async fn Set_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
        Value: &str,
    ) -> Result_type<()> {
        let Environment_variable = Environment_variable_type::New(Name, Value);

        // Keep the write lock for the entire operation
        let mut Inner = self.0.write().await;
        let Metadata = Self::Get_task_mutable(&mut Inner, Task_identifier)?;

        // We remove any existing environment variable with the same name
        Metadata
            .Environment_variables
            .retain(|Variable| Variable.Get_name() != Name);
        // Add the new environment variable
        Metadata.Environment_variables.push(Environment_variable);

        Ok(())
    }

    pub async fn Remove_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result_type<()> {
        Self::Get_task_mutable(&mut *self.0.write().await, Task_identifier)?
            .Environment_variables
            .retain(|Variable| Variable.Get_name() != Name);

        Ok(())
    }

    /// Get user identifier of the owner of a task.
    pub async fn Get_user(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<User_identifier_type> {
        Self::Get_task(&*self.0.read().await, Task_identifier).map(|Task| Task.User)
    }

    /// Get group identifier of the owner of a task.
    pub async fn Get_group(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Group_identifier_type> {
        Self::Get_task(&*self.0.read().await, Task_identifier).map(|Task| Task.Group)
    }

    pub async fn Get_environment_variable(
        &self,
        Task_identifier: Task_identifier_type,
        Name: &str,
    ) -> Result_type<Environment_variable_type> {
        Self::Get_task(&*self.0.read().await, Task_identifier)?
            .Environment_variables
            .iter()
            .find(|Variable| Variable.Get_name() == Name)
            .cloned()
            // If the variable is not found, return an error
            .ok_or(Error_type::Invalid_environment_variable)
    }

    pub async fn Get_environment_variables(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Environment_variable_type>> {
        Self::Get_task(&*self.0.read().await, Task_identifier)
            .map(|Task| Task.Environment_variables.clone())
            .map_err(|_| Error_type::Invalid_task_identifier)
    }

    /// # Arguments
    /// * `Task_identifier` - The identifier of the task.
    pub async fn Get_name(&self, Task_identifier: Task_identifier_type) -> Result_type<String> {
        Self::Get_task(&*self.0.read().await, Task_identifier).map(|Task| Task.Name.to_string())
    }
}
