use core::fmt::Write;
use core::str::from_utf8_unchecked;
use xila::{
    file_system::{Mode, Path},
    virtual_file_system,
};

use crate::{Error, Result, Shell};

impl Shell {
    async fn read_file_and_write(&mut self, path: &Path) -> Result<()> {
        let file = virtual_file_system::get_instance()
            .open(&path, Mode::READ_ONLY.into(), self.standard.get_task())
            .await
            .map_err(Error::FailedToOpenFile)?;

        let mut buffer = [0_u8; 128];
        while let Ok(size) = virtual_file_system::get_instance()
            .read(file, &mut buffer, self.standard.get_task())
            .await
        {
            if size == 0 {
                break;
            }

            let size: usize = size.into();

            let output = unsafe { from_utf8_unchecked(&buffer[..size]) };

            write!(self.standard.out(), "{}", output)?;
        }

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
