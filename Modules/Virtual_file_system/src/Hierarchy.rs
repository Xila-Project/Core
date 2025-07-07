/// Hierarchy of the file system.
use File_system::{Error_type, Path_type, Result_type, Type_type};
use Task::Task_identifier_type;

use crate::{Directory_type, Virtual_file_system_type};

/// Create the default hierarchy of the file system.
pub async fn Create_default_hierarchy(
    virtual_file_system: &Virtual_file_system_type<'_>,
    task: Task_identifier_type,
) -> Result_type<()> {
    virtual_file_system
        .Create_directory(&Path_type::SYSTEM, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::CONFIGURATION, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::SHARED_CONFIGURATION, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::DEVICES, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::USERS, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::DATA, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::SHARED_DATA, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::BINARIES, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::TEMPORARY, task)
        .await?;
    virtual_file_system
        .Create_directory(&Path_type::LOGS, task)
        .await?;

    Ok(())
}

pub async fn Clean_devices_in_directory<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
    path: &Path_type,
) -> Result_type<()> {
    // For each entry in the directory.
    for Entry in Directory_type::Open(virtual_file_system, path).await? {
        if Entry.Get_type() != Type_type::File {
            continue;
        }

        let Entry_path = path.Append(Entry.Get_name()).unwrap();

        let Metadata = virtual_file_system
            .Get_metadata_from_path(&Entry_path)
            .await?;

        if Metadata.Get_type() != Type_type::Character_device
            && Metadata.Get_type() != Type_type::Block_device
        {
            continue;
        }

        match virtual_file_system.Remove(&Entry_path).await {
            Ok(_) | Err(Error_type::Invalid_identifier) => {}
            Err(error) => {
                return Err(error);
            }
        }
    }

    Ok(())
}

pub async fn Clean_devices<'a>(
    virtual_file_system: &'a Virtual_file_system_type<'a>,
) -> Result_type<()> {
    Clean_devices_in_directory(virtual_file_system, Path_type::DEVICES).await?;

    Clean_devices_in_directory(virtual_file_system, Path_type::BINARIES).await?;

    Ok(())
}
