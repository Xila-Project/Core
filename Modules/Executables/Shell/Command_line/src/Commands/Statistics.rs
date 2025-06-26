use alloc::{borrow::ToOwned, format, string::ToString};
use File_system::{Inode_type, Path_type};

use crate::Shell_type;

impl Shell_type {
    pub async fn Statistics(&mut self, Arguments: &[&str]) {
        if Arguments.len() != 1 {
            self.Standard
                .Print_error_line("Invalid number of arguments")
                .await;
            return;
        }

        let Path = Path_type::From_str(Arguments[0]);

        let Path = if Path.Is_absolute() {
            Path.to_owned()
        } else {
            match self.Current_directory.clone().Join(Path) {
                Some(Path) => Path,
                None => {
                    self.Standard.Print_error_line("Invalid path").await;
                    return;
                }
            }
        };

        let Metadata = match Virtual_file_system::Get_instance()
            .Get_metadata_from_path(&Path)
            .await
        {
            Ok(Metadata) => Metadata,
            Err(Error) => {
                self.Standard.Print_error_line(&Error.to_string()).await;
                return;
            }
        };

        let User = match Users::Get_instance()
            .Get_user_name(Metadata.Get_user())
            .await
        {
            Ok(User) => User,
            Err(_) => {
                format!("{}", Metadata.Get_user().As_u16())
            }
        };

        let Group = match Users::Get_instance()
            .Get_group_name(Metadata.Get_group())
            .await
        {
            Ok(Group) => Group,
            Err(_) => {
                format!("{}", Metadata.Get_group().As_u16())
            }
        };

        let Inode = Metadata.Get_inode().unwrap_or(Inode_type::New(0)).As_u64();

        self.Standard
            .Print_line(&format!(
                r#"Type: {} - Inode : {}
User: {} - Group: {} - Permissions: {}
Accessed: {}
Modified: {}
Changed: {}"#,
                Metadata.Get_type(),
                Inode,
                User,
                Group,
                Metadata.Get_permissions(),
                Metadata.Get_access_time(),
                Metadata.Get_modification_time(),
                Metadata.Get_creation_time()
            ))
            .await;
    }
}
