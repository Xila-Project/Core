use crate::{Error, Result, Shell};
use xila::task;

impl Shell {
    pub async fn set_environment_variable(&mut self, arguments: &[&str]) -> Result<()> {
        if arguments.len() != 1 {
            return Err(Error::InvalidNumberOfArguments);
        }

        let (name, value) = arguments[0].split_once('=').ok_or(Error::InvalidArgument)?;

        task::get_instance()
            .set_environment_variable(self.task, name, value)
            .await
            .map_err(Error::FailedToSetEnvironmentVariable)
    }

    pub async fn remove_environment_variable(&mut self, arguments: &[&str]) -> Result<()> {
        if arguments.len() != 1 {
            return Err(Error::InvalidNumberOfArguments);
        }

        let name = arguments[0];

        task::get_instance()
            .remove_environment_variable(self.task, name)
            .await
            .map_err(Error::FailedToRemoveEnvironmentVariable)
    }
}
