use crate::{Error, Result, Shell, commands::check_no_more_arguments};
use getargs::Options;
use xila::task;

impl Shell {
    pub async fn set_environment_variable<'a, I>(
        &mut self,
        options: &mut Options<&'a str, I>,
    ) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let argument = options
            .next_positional()
            .ok_or(Error::MissingPositionalArgument("name=value"))?;

        check_no_more_arguments(options)?;

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
        let name = options
            .next_positional()
            .ok_or(Error::MissingPositionalArgument("name"))?;

        check_no_more_arguments(options)?;

        task::get_instance()
            .remove_environment_variable(self.task, name)
            .await
            .map_err(Error::FailedToRemoveEnvironmentVariable)
    }
}
