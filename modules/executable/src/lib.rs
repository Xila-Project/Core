#![no_std]

extern crate alloc;

#[cfg(feature = "building")]
mod building;
mod device_trait;
mod error;
mod read_data;
mod standard;

use alloc::string::String;
#[cfg(feature = "building")]
pub use building::*;
pub use device_trait::*;
pub use error::*;
pub use read_data::*;
pub use standard::*;

pub use file_system as exported_file_system;
pub use futures as exported_futures;
pub use task as exported_task;
pub use virtual_file_system as exported_virtual_file_system;

use task::{JoinHandle, TaskIdentifier};
use users::UserIdentifier;
use virtual_file_system::File;

use file_system::{Path, Statistics_type};

async fn is_execute_allowed(statistics: &Statistics_type, user: UserIdentifier) -> bool {
    // - Check if the file can executed by anyone
    if statistics.get_permissions().get_others().get_execute() {
        return true;
    }

    // - Check if the user is the owner and has the execute permission
    if user == UserIdentifier::ROOT {
        return true;
    }
    if (statistics.get_user() == user) && statistics.get_permissions().get_user().get_execute() {
        return true;
    }

    // - Check if the user is in the group
    let is_in_group = users::get_instance()
        .is_in_group(user, statistics.get_group())
        .await;

    // - Check if the user is in the group
    if (is_in_group) && statistics.get_permissions().get_group().get_execute() {
        return true;
    }

    false
}

async fn get_overridden_user(
    statistics: &Statistics_type,
    task: TaskIdentifier,
) -> Result<Option<UserIdentifier>> {
    if !statistics
        .get_permissions()
        .get_special()
        .get_set_user_identifier()
    {
        return Ok(None);
    }

    let current_user = task::get_instance().get_user(task).await?;

    let new_user = statistics.get_user();

    if current_user != users::UserIdentifier::ROOT || new_user != current_user {
        return Err(Error::PermissionDenied);
    }

    Ok(Some(new_user))
}

pub async fn execute(
    path: impl AsRef<Path>,
    inputs: String,
    standard: Standard,
) -> Result<JoinHandle<isize>> {
    let task_instance = task::get_instance();

    let task = task_instance.get_current_task_identifier().await;

    let file = File::open(
        virtual_file_system::get_instance(),
        &path,
        file_system::Mode::READ_WRITE.into(),
    )
    .await?;

    // - Check the executable bit
    if !is_execute_allowed(
        &file.get_statistics().await?,
        task_instance.get_user(task).await?,
    )
    .await
    {
        return Err(Error::PermissionDenied);
    }

    // - Check if the user can override the user identifier
    let new_user = get_overridden_user(&file.get_statistics().await?, task).await?;

    let file_name = path
        .as_ref()
        .get_file_name()
        .ok_or(file_system::Error::InvalidPath)?;

    let mut read_data = ReadData::new_default();
    file.read(&mut read_data).await?;
    let read_data: ReadData = read_data.try_into().unwrap();

    let main = read_data.get_main().ok_or(Error::FailedToGetMainFunction)?;

    let (join_handle, _) = task_instance
        .spawn(task, file_name, None, async move |task| {
            if let Some(new_user) = new_user {
                task::get_instance().set_user(task, new_user).await.unwrap();
            }

            let standard = standard.transfert(task).await.unwrap();

            match main(standard, inputs).await {
                Ok(_) => 0_isize,
                Err(error) => -(error.get() as isize),
            }
        })
        .await?;

    Ok(join_handle)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use file_system::Time;

    use task::test;

    use super::*;

    #[test]
    async fn is_user_allowed_test() {
        let statistics = Statistics_type::new(
            file_system::FileSystemIdentifier::new(0),
            file_system::Inode::new(0),
            1,
            0_usize.into(),
            Time::new(0),
            Time::new(0),
            Time::new(0),
            file_system::Kind::File,
            file_system::Permissions::from_octal(0o777).unwrap(),
            users::UserIdentifier::ROOT,
            users::GroupIdentifier::ROOT,
        );

        assert!(is_execute_allowed(&statistics, users::UserIdentifier::ROOT).await);
        assert!(is_execute_allowed(&statistics, users::UserIdentifier::ROOT).await);
        assert!(is_execute_allowed(&statistics, users::UserIdentifier::ROOT).await);
    }
}
