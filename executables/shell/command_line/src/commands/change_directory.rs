use alloc::borrow::ToOwned;
use xila::{file_system::Path, virtual_file_system};

use crate::{Error, Result, Shell};

impl Shell {
    pub async fn change_directory(&mut self, arguments: &[&str]) -> Result<()> {
        if arguments.len() != 1 {
            return Err(Error::InvalidNumberOfArguments);
        }

        let current_directory = Path::from_str(arguments[0]).to_owned();

        let current_directory = if current_directory.is_absolute() {
            current_directory
        } else {
            match self.current_directory.clone().join(&current_directory) {
                Some(path) => path.canonicalize(),
                None => {
                    return Err(Error::FailedToJoinPath);
                }
            }
        };
        virtual_file_system::get_instance()
            .open_directory(&current_directory, self.standard.get_task())
            .await
            .map_err(Error::FailedToOpenDirectory)?;

        self.current_directory = current_directory.to_owned();

        Ok(())
    }
}
