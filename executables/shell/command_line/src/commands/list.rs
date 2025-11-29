use crate::{Error, Result, Shell};
use alloc::string::ToString;
use core::fmt::Write;
use getargs::Arg;
use xila::{
    file_system::{Kind, Path},
    log, users,
    virtual_file_system::{self, Directory},
};

impl Shell {
    pub async fn list<'a, I>(&mut self, options: &mut getargs::Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut path: &Path = self.current_directory.as_ref();

        let mut long = false;

        while let Some(argument) = options.next_arg()? {
            match argument {
                Arg::Positional(p) => {
                    path = Path::from_str(p);
                }
                Arg::Short('l') | Arg::Long("long") => {
                    long = true;
                }
                _ => {
                    return Err(Error::InvalidOption);
                }
            }
        }

        let virtual_file_system = virtual_file_system::get_instance();

        let mut directory = Directory::open(virtual_file_system, self.task, &path)
            .await
            .map_err(Error::FailedToOpenDirectory)?;

        while let Some(entry) = directory
            .read()
            .await
            .map_err(Error::FailedToReadDirectoryEntry)?
        {
            if long {
                let entry_path = path
                    .join(Path::from_str(&entry.name))
                    .ok_or(Error::FailedToJoinPath)?;

                let statistics = virtual_file_system
                    .get_statistics(&entry_path)
                    .await
                    .map_err(Error::FailedToGetMetadata)?;

                let users_manager = users::get_instance();

                let user = users_manager
                    .get_user_name(statistics.user)
                    .await
                    .unwrap_or_else(|_| statistics.user.as_u16().to_string());

                let group = users_manager
                    .get_group_name(statistics.group)
                    .await
                    .unwrap_or_else(|_| statistics.group.as_u16().to_string());

                let kind = match statistics.kind {
                    Kind::File => "f",
                    Kind::Directory => "d",
                    Kind::CharacterDevice => "c",
                    Kind::BlockDevice => "b",
                    Kind::Pipe => "p",
                    Kind::Socket => "s",
                    Kind::SymbolicLink => "l",
                };

                writeln!(
                    self.standard.out(),
                    "{} {} {} {} {} {} {} {}",
                    kind,
                    statistics.permissions,
                    statistics.links,
                    user,
                    group,
                    statistics.size,
                    statistics.modification,
                    entry.name
                )?;
            } else {
                writeln!(self.standard.out(), "{}", entry.name)?;
            }
        }

        directory.close(virtual_file_system).await.map_err(|e| {
            log::error!("Failed to close directory {:?}", path);
            Error::FailedToOpenDirectory(e)
        })?;

        Ok(())
    }
}
