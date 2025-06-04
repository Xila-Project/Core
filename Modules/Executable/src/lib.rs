#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Device_trait;
mod Error;
mod Read_data;
mod Standard;

pub use Device_trait::*;
pub use Error::*;
pub use Read_data::*;
pub use Standard::*;

use Task::{Join_handle_type, Task_identifier_type};
use Users::User_identifier_type;
use Virtual_file_system::File_type;

use File_system::{Path_type, Statistics_type};

fn Is_execute_allowed(Statistics: &Statistics_type, User: User_identifier_type) -> bool {
    // - Check if the file can executed by anyone
    if Statistics.Get_permissions().Get_others().Get_execute() {
        return true;
    }

    // - Check if the user is the owner and has the execute permission
    if User == User_identifier_type::Root {
        return true;
    }
    if (Statistics.Get_user() == User) && Statistics.Get_permissions().Get_user().Get_execute() {
        return true;
    }

    // - Check if the user is in the group
    let Is_in_group = Users::Get_instance().Is_in_group(User, Statistics.Get_group());

    // - Check if the user is in the group
    if (Is_in_group) && Statistics.Get_permissions().Get_group().Get_execute() {
        return true;
    }

    false
}

async fn Get_overridden_user(
    Statistics: &Statistics_type,
    Task: Task_identifier_type,
) -> Result_type<Option<User_identifier_type>> {
    if !Statistics
        .Get_permissions()
        .Get_special()
        .Get_set_user_identifier()
    {
        return Ok(None);
    }

    let Current_user = Task::Get_instance().Get_user(Task).await?;

    let New_user = Statistics.Get_user();

    if Current_user != Users::User_identifier_type::Root || New_user != Current_user {
        return Err(Error_type::Permission_denied);
    }

    Ok(Some(New_user))
}

pub async fn Execute(
    Path: impl AsRef<Path_type>,
    Inputs: String,
    Standard: Standard_type,
) -> Result_type<Join_handle_type<isize>> {
    let Task_instance = Task::Get_instance();

    let Task = Task_instance.Get_current_task_identifier().await;

    let File = File_type::Open(
        Virtual_file_system::Get_instance().await,
        &Path,
        File_system::Mode_type::Read_write.into(),
    )
    .await?;

    // - Check the executable bit
    if !Is_execute_allowed(
        &File.Get_statistics().await?,
        Task_instance.Get_user(Task).await?,
    ) {
        return Err(Error_type::Permission_denied);
    }

    // - Check if the user can override the user identifier
    let New_user = Get_overridden_user(&File.Get_statistics().await?, Task).await?;

    let File_name = Path
        .as_ref()
        .Get_file_name()
        .ok_or(File_system::Error_type::Invalid_path)?;

    let mut Read_data = Read_data_type::New_default();
    File.Read(&mut Read_data).await?;
    let Read_data: Read_data_type = Read_data.try_into().unwrap();

    let Main = Read_data
        .Get_main()
        .ok_or(Error_type::Failed_to_get_main_function)?;

    let (Join_handle, _) = Task_instance
        .Spawn(Task, File_name, None, async move |Task| {
            if let Some(New_user) = New_user {
                Task::Get_instance().Set_user(Task, New_user).await.unwrap();
            }

            let Standard = Standard.Transfert(Task).await.unwrap();

            match Main(Standard, Inputs) {
                Ok(_) => 0_isize,
                Err(Error) => -(Error.get() as isize),
            }
        })
        .await?;

    Ok(Join_handle)
}

#[cfg(test)]
mod Tests {
    use File_system::Time_type;

    use super::*;

    #[test]
    fn Is_user_allowed_test() {
        let Statistics = Statistics_type::New(
            File_system::File_system_identifier_type::New(0),
            File_system::Inode_type::New(0),
            1,
            0_usize.into(),
            Time_type::New(0),
            Time_type::New(0),
            Time_type::New(0),
            File_system::Type_type::File,
            File_system::Permissions_type::From_octal(0o777).unwrap(),
            Users::User_identifier_type::Root,
            Users::Group_identifier_type::Root,
        );

        assert!(Is_execute_allowed(
            &Statistics,
            Users::User_identifier_type::Root
        ));
        assert!(Is_execute_allowed(
            &Statistics,
            Users::User_identifier_type::Root
        ));
        assert!(Is_execute_allowed(
            &Statistics,
            Users::User_identifier_type::Root
        ));
    }
}
