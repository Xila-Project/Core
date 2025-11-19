use core::fmt::Write;
use xila::{
    file_system::{AccessFlags, Path},
    virtual_file_system::{self, File},
};

use crate::{Error, Result, Shell};

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

    pub async fn concatenate(&mut self, arguments: &[&str]) -> Result<()> {
        for path in arguments {
            let path = Path::from_str(path);

            if path.is_absolute() {
                self.read_file_and_write(path).await?;
            } else {
                match self.current_directory.clone().join(path) {
                    Some(path) => self.read_file_and_write(&path).await?,
                    None => {
                        return Err(Error::FailedToJoinPath);
                    }
                }
            }
        }

        writeln!(self.standard.out())?;

        Ok(())
    }
}
