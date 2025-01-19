#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod Error;
mod User;

pub use Error::*;
use File_system::{Flags_type, Mode_type, Path_owned_type, Path_type};
use Virtual_file_system::Virtual_file_system_type;

const Users_folder_path: &str = "/Xila/Users";

pub fn Load_all_users() -> Result_type<()> {
    // Open Xila users folder.
    let Virtual_file_system = Virtual_file_system::Get_instance();

    let Task_manager = Task::Get_instance();

    let Users_manager = Users::Get_instance();

    let Task = Task_manager.Get_current_task_identifier()?;

    let Users_folder = Virtual_file_system
        .Open_directory(&Users_folder_path, Task)
        .map_err(Error_type::Failed_to_read_users_folder)?;

    // Read all users.
    while let Some(User_entry) = Virtual_file_system
        .Read_directory(Users_folder, Task)
        .map_err(Error_type::Failed_to_read_users_folder)?
    {
        let User_file_path = Path_type::New(Users_folder_path)
            .to_owned()
            .Append(User_entry.Get_name())
            .ok_or(Error_type::Failed_to_get_user_file_path)?;

        let User_file = Virtual_file_system
            .Open(&User_file_path, Mode_type::Read_only.into(), Task)
            .map_err(Error_type::Failed_to_read_users_folder)?;
    }

    Ok(())
}
