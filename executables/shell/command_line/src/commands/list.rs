use crate::{Error, Result, Shell};
use core::fmt::Write;
use xila::{file_system::Path, virtual_file_system};

impl Shell {
    pub async fn list(&mut self, arguments: &[&str]) -> Result<()> {
        let path = if arguments.is_empty() {
            self.current_directory.as_ref()
        } else {
            Path::from_str(arguments[0])
        };

        let directory = virtual_file_system::get_instance()
            .open_directory(&path, self.standard.get_task())
            .await
            .map_err(Error::FailedToOpenDirectory)?;

        while let Some(entry) = virtual_file_system::get_instance()
            .read_directory(directory, self.standard.get_task())
            .await
            .map_err(Error::FailedToReadDirectoryEntry)?
        {
            writeln!(self.standard.out(), "{}", entry.get_name())?;
        }

        Ok(())
    }
}
