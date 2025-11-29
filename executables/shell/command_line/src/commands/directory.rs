use crate::{Error, Result, Shell, commands::check_no_more_arguments};
use alloc::borrow::ToOwned;
use getargs::Options;
use xila::{
    file_system::Path,
    virtual_file_system::{self, Directory},
};

impl Shell {
    pub async fn create_directory<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let path = options
            .next_positional()
            .ok_or(Error::MissingPositionalArgument("path"))?;

        check_no_more_arguments(options)?;

        let path = Path::from_str(path);

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

        Directory::create(virtual_file_system::get_instance(), self.task, &path)
            .await
            .map_err(Error::FailedToCreateDirectory)
    }

    pub async fn remove<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let path = options
            .next_positional()
            .ok_or(Error::MissingPositionalArgument("path"))?;
        let path = Path::from_str(path);

        check_no_more_arguments(options)?;

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
            .remove(self.task, &path)
            .await
            .map_err(Error::FailedToRemoveDirectory)
    }
}
