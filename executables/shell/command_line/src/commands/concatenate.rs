use core::fmt::Write;
use getargs::Options;
use xila::{
    file_system::{AccessFlags, Path},
    virtual_file_system::{self, File},
};

use crate::{Error, Result, Shell, commands::check_no_more_options};

impl Shell {
    async fn read_file_and_write(&mut self, path: &Path) -> Result<()> {
        let virtual_file_system = virtual_file_system::get_instance();

        let mut file = File::open(
            virtual_file_system,
            self.task,
            &path,
            AccessFlags::Read.into(),
        )
        .await
        .map_err(Error::FailedToOpenFile)?;

        let _ = file.display_content(self.standard.out(), 256).await;

        Ok(())
    }

    pub async fn concatenate<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        check_no_more_options(options)?;

        while let Some(argument) = options.next_positional() {
            let path = Path::from_str(argument);

            if path.is_absolute() {
                self.read_file_and_write(path).await?;
            } else {
                let path = self
                    .current_directory
                    .clone()
                    .join(path)
                    .ok_or(Error::FailedToJoinPath)?;

                self.read_file_and_write(&path).await?;
            }
        }

        writeln!(self.standard.out())?;

        Ok(())
    }
}
