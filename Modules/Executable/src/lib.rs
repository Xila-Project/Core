#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

mod device_trait;
mod error;
mod read_data;
mod standard;

use alloc::string::String;
pub use device_trait::*;
pub use error::*;
pub use read_data::*;
pub use standard::*;

use task::{Join_handle_type, Task_identifier_type};
use users::User_identifier_type;
use virtual_file_system::File_type;

use file_system::{Path_type, Statistics_type};

async fn is_execute_allowed(Statistics: &Statistics_type, User: User_identifier_type) -> bool {
    // - Check if the file can executed by anyone
    if Statistics.get_permissions().get_others().get_execute() {
        return true;
    }

    // - Check if the user is the owner and has the execute permission
    if User == User_identifier_type::ROOT {
        return true;
    }
    if (Statistics.get_user() == User) && Statistics.get_permissions().get_user().get_execute() {
        return true;
    }

    // - Check if the user is in the group
    let is_in_group = users::get_instance()
        .is_in_group(User, Statistics.get_group())
        .await;

    // - Check if the user is in the group
    if (is_in_group) && Statistics.get_permissions().get_group().get_execute() {
        return true;
    }

    false
}

async fn get_overridden_user(
    statistics: &Statistics_type,
    task: Task_identifier_type,
) -> Result_type<Option<User_identifier_type>> {
    if !statistics
        .get_permissions()
        .get_special()
        .get_set_user_identifier()
    {
        return Ok(None);
    }

    let Current_user = task::get_instance().get_user(task).await?;

    let New_user = statistics.get_user();

    if Current_user != users::User_identifier_type::ROOT || New_user != Current_user {
        return Err(Error_type::Permission_denied);
    }

    Ok(Some(New_user))
}

pub async fn execute(
    path: impl AsRef<Path_type>,
    inputs: String,
    standard: Standard_type,
) -> Result_type<Join_handle_type<isize>> {
    let task_instance = task::get_instance();

    let Task = task_instance.get_current_task_identifier().await;

    let File = File_type::open(
        virtual_file_system::get_instance(),
        &path,
        file_system::Mode_type::READ_WRITE.into(),
    )
    .await?;

    // - Check the executable bit
    if !is_execute_allowed(
        &File.get_statistics().await?,
        task_instance.get_user(Task).await?,
    )
    .await
    {
        return Err(Error_type::Permission_denied);
    }

    // - Check if the user can override the user identifier
    let New_user = get_overridden_user(&File.get_statistics().await?, Task).await?;

    let File_name = path
        .as_ref()
        .get_file_name()
        .ok_or(file_system::Error_type::Invalid_path)?;

    let mut Read_data = Read_data_type::New_default();
    File.read(&mut Read_data).await?;
    let read_data: Read_data_type = Read_data.try_into().unwrap();

    let Main = read_data
        .get_main()
        .ok_or(Error_type::Failed_to_get_main_function)?;

    let (Join_handle, _) = task_instance
        .Spawn(Task, File_name, None, async move |Task| {
            if let Some(new_user) = New_user {
                task::get_instance().set_user(Task, new_user).await.unwrap();
            }

            let Standard = standard.transfert(Task).await.unwrap();

            match Main(Standard, inputs).await {
                Ok(_) => 0_isize,
                Err(error) => -(error.get() as isize),
            }
        })
        .await?;

    Ok(Join_handle)
}

#[cfg(test)]
mod tests {
    use file_system::Time_type;

    use task::Test;

    use super::*;

    #[Test]
    async fn is_user_allowed_test() {
        let Statistics = Statistics_type::new(
            file_system::File_system_identifier_type::New(0),
            file_system::Inode_type::New(0),
            1,
            0_usize.into(),
            Time_type::New(0),
            Time_type::New(0),
            Time_type::New(0),
            file_system::Type_type::File,
            file_system::Permissions_type::From_octal(0o777).unwrap(),
            users::User_identifier_type::ROOT,
            users::Group_identifier_type::ROOT,
        );

        assert!(is_execute_allowed(&Statistics, users::User_identifier_type::ROOT).await);
        assert!(is_execute_allowed(&Statistics, users::User_identifier_type::ROOT).await);
        assert!(is_execute_allowed(&Statistics, users::User_identifier_type::ROOT).await);
    }
}
