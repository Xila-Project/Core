use crate::{Error, Result, Shell};
use core::fmt::Write;
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

#[derive(GetArgs)]
struct GetEnvironmentVariableArguments<'a> {
    #[arg(positional, default = "")]
    key: &'a str,
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

    pub async fn print_environment_variable<'a, I>(
        &mut self,
        options: &mut getargs::Options<&'a str, I>,
    ) -> crate::Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let GetEnvironmentVariableArguments { key } =
            GetEnvironmentVariableArguments::parse(options)?;

        let task_manager = task::get_instance();

        if key.is_empty() {
            let environment_variables = task_manager
                .get_environment_variables(self.task)
                .await
                .map_err(|_| crate::Error::FailedToGetTaskIdentifier)?;

            for environment_variable in environment_variables {
                writeln!(
                    self.standard.out(),
                    "{}={}",
                    environment_variable.get_name(),
                    environment_variable.get_value()
                )?;
            }
        } else {
            let environment_variable = task_manager
                .get_environment_variable(self.task, key)
                .await
                .map_err(|_| crate::Error::FailedToGetTaskIdentifier)?;

            writeln!(
                self.standard.out(),
                "{}={}",
                environment_variable.get_name(),
                environment_variable.get_value()
            )?;
        }

        Ok(())
    }
}
