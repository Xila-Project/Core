use crate::error::Error;
use crate::{Result, Shell};
use core::fmt::Write;
use getargs::Options;
use xila::task;

impl Shell {
    pub async fn echo<'a, I>(&mut self, arguments: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        while let Some(argument) = arguments.next_positional() {
            if let Some(name) = argument.strip_prefix('$') {
                let environment_variable = task::get_instance()
                    .get_environment_variable(self.task, name)
                    .await
                    .map_err(Error::FailedToReadEnvironmentVariable)?;

                write!(self.standard.out(), "{} ", environment_variable.get_value())?;
            } else {
                let argument = argument.trim_matches('\"');
                write!(self.standard.out(), "{} ", argument)?;
            }
        }
        writeln!(self.standard.out())?;

        Ok(())
    }
}
