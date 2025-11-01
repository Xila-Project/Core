use crate::{Error, Result, Shell};
use alloc::borrow::ToOwned;
use xila::{file_system::Path, virtual_file_system};

impl Shell {
    pub async fn create_directory(&mut self, arguments: &[&str]) -> Result<()> {
        if arguments.len() != 1 {
            return Err(Error::InvalidNumberOfArguments);
        }

        let path = Path::from_str(arguments[0]);

        if !path.is_valid() {
            return Err(Error::InvalidPath);
        }

        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            match self.current_directory.clone().join(path) {
                Some(path) => path.canonicalize(),
                None => {
                    return Err(Error::FailedToJoinPath);
                }
            }
        };

        virtual_file_system::get_instance()
            .create_directory(&path, self.standard.get_task())
            .await
            .map_err(Error::FailedToCreateDirectory)
    }

    pub async fn remove(&mut self, arguments: &[&str]) -> Result<()> {
        if arguments.len() != 1 {
            return Err(Error::InvalidNumberOfArguments);
        }

        let path = Path::from_str(arguments[0]);

        if !path.is_valid() {
            return Err(Error::InvalidPath);
        }

        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            match self.current_directory.clone().join(path) {
                Some(path) => path.canonicalize(),
                None => {
                    return Err(Error::FailedToJoinPath);
                }
            }
        };

        virtual_file_system::get_instance()
            .remove(&path)
            .await
            .map_err(Error::FailedToRemoveDirectory)
    }
}
