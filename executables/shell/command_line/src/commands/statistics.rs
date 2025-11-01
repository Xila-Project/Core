use crate::{Error, Result, Shell};
use alloc::{borrow::ToOwned, format};
use core::fmt::Write;
use xila::{
    file_system::{Inode, Path},
    internationalization::translate,
    users, virtual_file_system,
};

impl Shell {
    pub async fn statistics(&mut self, arguments: &[&str]) -> Result<()> {
        if arguments.len() != 1 {
            return Err(Error::InvalidNumberOfArguments);
        }

        let path = Path::from_str(arguments[0]);

        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            self.current_directory
                .clone()
                .join(path)
                .ok_or(Error::FailedToJoinPath)?
        };

        let metadata = virtual_file_system::get_instance()
            .get_metadata_from_path(&path)
            .await
            .map_err(Error::FailedToGetMetadata)?;

        let user = match users::get_instance()
            .get_user_name(metadata.get_user())
            .await
        {
            Ok(user) => user,
            Err(_) => {
                format!("{}", metadata.get_user().as_u16())
            }
        };

        let group = match users::get_instance()
            .get_group_name(metadata.get_group())
            .await
        {
            Ok(group) => group,
            Err(_) => {
                format!("{}", metadata.get_group().as_u16())
            }
        };

        let inode = metadata.get_inode().unwrap_or(Inode::new(0)).as_u64();

        let _ = writeln!(
            self.standard.out(),
            r#"{}: {} - {} : {}
{}: {} - {}: {} - {}: {}
{}: {}
{}: {}
{}: {}"#,
            translate!("Type"),
            metadata.get_type(),
            translate!("Inode"),
            inode,
            translate!("User"),
            user,
            translate!("Group"),
            group,
            translate!("Permissions"),
            metadata.get_permissions(),
            translate!("Accessed"),
            metadata.get_access_time(),
            translate!("Modified"),
            metadata.get_modification_time(),
            translate!("Created"),
            metadata.get_creation_time()
        );

        Ok(())
    }
}
