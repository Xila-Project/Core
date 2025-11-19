use crate::{Error, Result, Shell};
use core::fmt::Write;
use xila::{
    file_system::Path,
    log,
    virtual_file_system::{self, Directory},
};

impl Shell {
    pub async fn list(&mut self, arguments: &[&str]) -> Result<()> {
        let path = if arguments.is_empty() {
            self.current_directory.as_ref()
        } else {
            Path::from_str(arguments[0])
        };

        let virtual_file_system = virtual_file_system::get_instance();

        let mut directory = Directory::open(virtual_file_system, self.task, &path)
            .await
            .map_err(Error::FailedToOpenDirectory)?;

        while let Some(entry) = directory
            .read()
            .await
            .map_err(Error::FailedToReadDirectoryEntry)?
        {
            writeln!(self.standard.out(), "{}", entry.name)?;
        }

        directory.close(virtual_file_system).await.map_err(|e| {
            log::error!("Failed to close directory {:?}", path);
            Error::FailedToOpenDirectory(e)
        })?;

        Ok(())
    }
}
