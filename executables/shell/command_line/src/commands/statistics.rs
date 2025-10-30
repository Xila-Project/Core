use alloc::{borrow::ToOwned, format};

use xila::{
    file_system::{Inode, Path},
    users, virtual_file_system,
};

use crate::{Error, Result, Shell};

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

        self.standard
            .print_line(&format!(
                r#"Type: {} - Inode : {}
User: {} - Group: {} - Permissions: {}
Accessed: {}
Modified: {}
Changed: {}"#,
                metadata.get_type(),
                inode,
                user,
                group,
                metadata.get_permissions(),
                metadata.get_access_time(),
                metadata.get_modification_time(),
                metadata.get_creation_time()
            ))
            .await;

        Ok(())
    }
}
