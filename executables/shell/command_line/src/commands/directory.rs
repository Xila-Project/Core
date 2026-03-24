use crate::{Error, Result, Shell};
use alloc::borrow::ToOwned;
use executable_macros::GetArgs;
use getargs::Options;
use xila::{
    file_system::Path,
    virtual_file_system::{self, Directory},
};

#[derive(GetArgs)]
struct DirectoryCreateArguments<'a> {
    path: &'a str,
}

#[derive(GetArgs)]
struct DirectoryRemoveArguments<'a> {
    path: &'a str,
}

impl Shell {
    pub async fn create_directory<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let DirectoryCreateArguments { path } = DirectoryCreateArguments::parse(options)?;

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
        let DirectoryRemoveArguments { path } = DirectoryRemoveArguments::parse(options)?;
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

        virtual_file_system::get_instance()
            .remove(self.task, &path)
            .await
            .map_err(Error::FailedToRemoveDirectory)
    }
}
