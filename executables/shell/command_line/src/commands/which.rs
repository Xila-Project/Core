use core::fmt::Write;
use executable_macros::GetArgs;
use getargs::Options;
use xila::{file_system::Path, virtual_file_system};

use crate::{Result, Shell, error::Error, resolver::resolve};

#[derive(GetArgs)]
struct WhichArguments<'a> {
    command: &'a str,
}

impl Shell {
    async fn print_result(&mut self, resolved_path: &Path) -> Result<()> {
        let _ = virtual_file_system::get_instance()
            .get_statistics(&resolved_path)
            .await
            .map_err(Error::FailedToGetMetadata)?;

        writeln!(self.standard.out(), "{}", resolved_path.as_str())?;
        Ok(())
    }

    pub async fn which<'a, I>(
        &mut self,
        options: &mut Options<&'a str, I>,
        paths: &[&Path],
    ) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let WhichArguments { command } = WhichArguments::parse(options)?;

        let path = Path::from_str(command);

        if path.is_valid() {
            if path.is_absolute() {
                self.print_result(&path).await?;
            } else {
                let path = self
                    .current_directory
                    .clone()
                    .join(path)
                    .ok_or(Error::FailedToJoinPath)?;

                self.print_result(&path).await?;
            }
        } else {
            let path = resolve(path.as_str(), paths).await?;

            self.print_result(&path).await?;
        }

        Ok(())
    }
}
