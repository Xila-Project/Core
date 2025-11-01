use crate::{
    Shell,
    error::{Error, Result},
    parser::Command,
    resolver::resolve,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use xila::{executable::execute, file_system::Path, task};

impl Shell {
    pub async fn execute<'a>(&mut self, command: Command<'a>, paths: &[&Path]) -> Result<()> {
        // - Set the current directory for the following commands.
        task::get_instance()
            .set_environment_variable(
                self.standard.get_task(),
                "Current_directory",
                self.current_directory.as_str(),
            )
            .await
            .map_err(Error::FailedToSetCurrentDirectory)?;

        let path = Path::from_str(command.command);

        if path.is_valid() {
            if path.is_absolute() {
                self.run(path, command.arguments).await?;
            } else {
                let path = self
                    .current_directory
                    .clone()
                    .join(path)
                    .ok_or(Error::FailedToJoinPath)?;

                self.run(&path, command.arguments).await?;
            }
        } else {
            let path = resolve(command.command, paths).await?;

            self.run(&path, command.arguments).await?;
        }

        Ok(())
    }

    async fn run<'a, I>(&mut self, path: &Path, arguments: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let standard = self.standard.duplicate().await.unwrap();

        let arguments: Vec<String> = arguments.into_iter().map(|s| s.to_string()).collect();

        let _ = execute(path, arguments, standard, None)
            .await
            .map_err(|_| Error::FailedToExecuteCommand)?
            .join()
            .await;

        Ok(())
    }
}
