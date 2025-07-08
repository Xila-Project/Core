/// Hierarchy of the file system.
use file_system::{Error_type, Path_type, Result_type, Type_type};
use task::Task_identifier_type;

use crate::{Directory_type, Virtual_file_system_type};

/// Create the default hierarchy of the file system.
pub async fn create_default_hierarchy(
    virtual_file_system: &Virtual_file_system_type<'_>,
    task: Task_identifier_type,
) -> Result_type<()> {
    virtual_file_system
        .create_directory(&Path_type::SYSTEM, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::CONFIGURATION, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::SHARED_CONFIGURATION, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::DEVICES, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::USERS, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::DATA, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::SHARED_DATA, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::BINARIES, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::TEMPORARY, task)
        .await?;
    virtual_file_system
        .create_directory(&Path_type::LOGS, task)
        .await?;

    Ok(())
}

pub async fn clean_devices_in_directory<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
    path: &Path_type,
) -> Result_type<()> {
    // For each entry in the directory.
    for entry in Directory_type::open(virtual_file_system, path).await? {
        if entry.get_type() != Type_type::File {
            continue;
        }

        let entry_path = path.Append(entry.get_name()).unwrap();

        if virtual_file_system
            .get_metadata_from_path(&entry_path)
            .await?
            .get_type()
            != Type_type::Character_device
            && virtual_file_system
                .get_metadata_from_path(&entry_path)
                .await?
                .get_type()
                != Type_type::Block_device
        {
            continue;
        }

        match virtual_file_system.remove(&entry_path).await {
            Ok(_) | Err(Error_type::Invalid_identifier) => {}
            Err(error) => {
                return Err(error);
            }
        }
    }

    Ok(())
}

pub async fn clean_devices<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
) -> Result_type<()> {
    clean_devices_in_directory(virtual_file_system, Path_type::DEVICES).await?;

    clean_devices_in_directory(virtual_file_system, Path_type::BINARIES).await?;

    Ok(())
}
