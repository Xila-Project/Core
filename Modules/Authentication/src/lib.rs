#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod Error;
mod Group;
mod Hash;
mod User;

pub use Error::*;
pub use Group::*;
pub use User::*;

const Users_folder_path: &str = "/System/Users";
const Group_folder_path: &str = "/System/Groups";
const Random_device_path: &str = "/Devices/Random";

pub async fn Load_all_users_and_groups() -> Result_type<()> {
    use Group::Read_group_file;
    use User::Read_user_file;
    use Virtual_file_system::Directory_type;
    // Open Xila users folder.
    let Virtual_file_system = &Virtual_file_system::Get_instance().await;

    let Users_manager = Users::Get_instance().await;

    let mut Buffer: Vec<u8> = vec![];

    {
        let Groups_directory = Directory_type::Open(Virtual_file_system, Group_folder_path)
            .await
            .map_err(Error_type::Failed_to_read_group_directory)?;

        // Read all groups.
        for Group_entry in Groups_directory {
            let Group = if let Ok(Group) =
                Read_group_file(Virtual_file_system, &mut Buffer, Group_entry.Get_name()).await
            {
                Group
            } else {
                // ? : Log error ?
                continue;
            };

            Users_manager
                .Add_group(Group.Get_identifier(), Group.Get_name(), Group.Get_users())
                .await
                .map_err(Error_type::Failed_to_add_group)?;
        }
    }

    {
        let Users_directory = Directory_type::Open(Virtual_file_system, Users_folder_path)
            .await
            .map_err(Error_type::Failed_to_read_users_directory)?;

        // Read all users.
        for User_entry in Users_directory {
            let User = if let Ok(User) =
                Read_user_file(Virtual_file_system, &mut Buffer, User_entry.Get_name()).await
            {
                User
            } else {
                // ? : Log error ?
                continue;
            };

            Users_manager
                .Add_user(
                    User.Get_identifier(),
                    User.Get_name(),
                    User.Get_primary_group(),
                )
                .await
                .map_err(Error_type::Failed_to_add_user)?;
        }
    }

    Ok(())
}
