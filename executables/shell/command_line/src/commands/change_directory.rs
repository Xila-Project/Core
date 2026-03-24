use alloc::borrow::ToOwned;
use executable_macros::GetArgs;
use xila::{
    file_system::Path,
    virtual_file_system::{self, Directory},
};

use crate::{Error, Result, Shell};

#[derive(GetArgs)]
struct ChangeDirectoryArguments<'a> {
    path: &'a str,
}

impl Shell {
    pub async fn change_directory<'a, I>(
        &mut self,
        options: &mut getargs::Options<&'a str, I>,
    ) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let ChangeDirectoryArguments { path } = ChangeDirectoryArguments::parse(options)?;

        let current_directory = Path::from_str(path).to_owned();

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

        let virtual_file_system = virtual_file_system::get_instance();

        let _ = Directory::open(virtual_file_system, self.task, &current_directory)
            .await
            .map_err(Error::FailedToOpenDirectory)?
            .close(virtual_file_system)
            .await;

        self.current_directory = current_directory.to_owned();

        Ok(())
    }
}
