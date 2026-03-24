use crate::{Error, Result, Shell};
use executable_macros::GetArgs;
use getargs::Options;
use xila::task;

#[derive(GetArgs)]
struct SetEnvironmentVariableArguments<'a> {
    argument: &'a str,
}

#[derive(GetArgs)]
struct RemoveEnvironmentVariableArguments<'a> {
    name: &'a str,
}

impl Shell {
    pub async fn set_environment_variable<'a, I>(
        &mut self,
        options: &mut Options<&'a str, I>,
    ) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let SetEnvironmentVariableArguments { argument } =
            SetEnvironmentVariableArguments::parse(options)?;

        let (name, value) = argument.split_once('=').ok_or(Error::InvalidArgument)?;

        task::get_instance()
            .set_environment_variable(self.task, name, value)
            .await
            .map_err(Error::FailedToSetEnvironmentVariable)
    }

    pub async fn remove_environment_variable<'a, I>(
        &mut self,
        options: &mut Options<&'a str, I>,
    ) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let RemoveEnvironmentVariableArguments { name } =
            RemoveEnvironmentVariableArguments::parse(options)?;

        task::get_instance()
            .remove_environment_variable(self.task, name)
            .await
            .map_err(Error::FailedToRemoveEnvironmentVariable)
    }
}
