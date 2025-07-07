/// Hierarchy of the file system.
use File_system::{Error_type, Path_type, Result_type, Type_type};
use Task::Task_identifier_type;

use crate::{Directory_type, Virtual_file_system_type};

/// Create the default hierarchy of the file system.
pub async fn Create_default_hierarchy(
    Virtual_file_system: &Virtual_file_system_type<'_>,
    Task: Task_identifier_type,
) -> Result_type<()> {
    Virtual_file_system
        .Create_directory(&Path_type::SYSTEM, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::CONFIGURATION, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::SHARED_CONFIGURATION, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::DEVICES, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::USERS, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::DATA, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::SHARED_DATA, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::BINARIES, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::TEMPORARY, Task)
        .await?;
    Virtual_file_system
        .Create_directory(&Path_type::LOGS, Task)
        .await?;

    Ok(())
}

pub async fn Clean_devices_in_directory<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Path: &Path_type,
) -> Result_type<()> {
    // For each entry in the directory.
    for Entry in Directory_type::Open(Virtual_file_system, Path).await? {
        if Entry.Get_type() != Type_type::File {
            continue;
        }

        let Entry_path = Path.Append(Entry.Get_name()).unwrap();

        let Metadata = Virtual_file_system
            .Get_metadata_from_path(&Entry_path)
            .await?;

        if Metadata.Get_type() != Type_type::Character_device
            && Metadata.Get_type() != Type_type::Block_device
        {
            continue;
        }

        match Virtual_file_system.Remove(&Entry_path).await {
            Ok(_) | Err(Error_type::Invalid_identifier) => {}
            Err(Error) => {
                return Err(Error);
            }
        }
    }

    Ok(())
}

pub async fn Clean_devices<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
) -> Result_type<()> {
    Clean_devices_in_directory(Virtual_file_system, Path_type::DEVICES).await?;

    Clean_devices_in_directory(Virtual_file_system, Path_type::BINARIES).await?;

    Ok(())
}
