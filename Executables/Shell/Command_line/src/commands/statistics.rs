use alloc::{borrow::ToOwned, format, string::ToString};

use file_system::{Inode_type, Path_type};

use crate::Shell_type;

impl Shell_type {
    pub async fn statistics(&mut self, arguments: &[&str]) {
        if arguments.len() != 1 {
            self.standard
                .print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let path = Path_type::from_str(arguments[0]);

        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            match self.current_directory.clone().join(path) {
                Some(path) => path,
                None => {
                    self.standard.print_error_line("Invalid path").await;
                    return;
                }
            }
        };

        let metadata = match virtual_file_system::get_instance()
            .get_metadata_from_path(&path)
            .await
        {
            Ok(metadata) => metadata,
            Err(error) => {
                self.standard.print_error_line(&error.to_string()).await;
                return;
            }
        };

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

        let inode = metadata.get_inode().unwrap_or(Inode_type::new(0)).as_u64();

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
    }
}
