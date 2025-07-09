// Properties module - handles task properties like user, group, and environment variables

use alloc::string::ToString;

use super::*;
use alloc::{string::String, vec::Vec};
use users::{Group_identifier_type, User_identifier_type};

impl Manager_type {
    pub async fn set_user(
        &self,
        task_identifier: Task_identifier_type,
        user: User_identifier_type,
    ) -> Result_type<()> {
        Self::get_task_mutable(&mut *self.0.write().await, task_identifier)?.user = user;

        Ok(())
    }

    pub async fn set_group(
        &self,
        task_identifier: Task_identifier_type,
        group: Group_identifier_type,
    ) -> Result_type<()> {
        Self::get_task_mutable(&mut *self.0.write().await, task_identifier)?.group = group;

        Ok(())
    }

    pub async fn set_environment_variable(
        &self,
        task_identifier: Task_identifier_type,
        name: &str,
        value: &str,
    ) -> Result_type<()> {
        let environment_variable = Environment_variable_type::new(name, value);

        // Keep the write lock for the entire operation
        let mut inner = self.0.write().await;
        let metadata = Self::get_task_mutable(&mut inner, task_identifier)?;

        // We remove any existing environment variable with the same name
        metadata
            .environment_variables
            .retain(|variable| variable.get_name() != name);
        // Add the new environment variable
        metadata.environment_variables.push(environment_variable);

        Ok(())
    }

    pub async fn set_environment_variables(
        &self,
        task_identifier: Task_identifier_type,
        environment_variables: &[(&str, &str)],
    ) -> Result_type<()> {
        let mut inner = self.0.write().await;
        let metadata = Self::get_task_mutable(&mut inner, task_identifier)?;

        environment_variables.iter().for_each(|(name, value)| {
            let environment_variable = Environment_variable_type::new(name, value);
            metadata.environment_variables.push(environment_variable);
        });

        Ok(())
    }

    pub async fn remove_environment_variable(
        &self,
        task_identifier: Task_identifier_type,
        name: &str,
    ) -> Result_type<()> {
        Self::get_task_mutable(&mut *self.0.write().await, task_identifier)?
            .environment_variables
            .retain(|variable| variable.get_name() != name);

        Ok(())
    }

    /// Get user identifier of the owner of a task.
    pub async fn get_user(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<User_identifier_type> {
        Self::get_task(&*self.0.read().await, task_identifier).map(|task| task.user)
    }

    /// Get group identifier of the owner of a task.
    pub async fn get_group(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Group_identifier_type> {
        Self::get_task(&*self.0.read().await, task_identifier).map(|task| task.group)
    }

    pub async fn get_environment_variable(
        &self,
        task_identifier: Task_identifier_type,
        name: &str,
    ) -> Result_type<Environment_variable_type> {
        Self::get_task(&*self.0.read().await, task_identifier)?
            .environment_variables
            .iter()
            .find(|variable| variable.get_name() == name)
            .cloned()
            // If the variable is not found, return an error
            .ok_or(Error_type::Invalid_environment_variable)
    }

    pub async fn get_environment_variables(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Vec<Environment_variable_type>> {
        Self::get_task(&*self.0.read().await, task_identifier)
            .map(|task| task.environment_variables.clone())
            .map_err(|_| Error_type::Invalid_task_identifier)
    }

    /// # Arguments
    /// * `Task_identifier` - The identifier of the task.
    pub async fn get_name(&self, task_identifier: Task_identifier_type) -> Result_type<String> {
        Self::get_task(&*self.0.read().await, task_identifier).map(|task| task.name.to_string())
    }
}
