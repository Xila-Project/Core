/// Hierarchy of the file system.
use file_system::{Error, Kind, Path, Result};
use task::TaskIdentifier;

use crate::{Directory, VirtualFileSystem};

/// Create the default hierarchy of the file system.
pub async fn create_default_hierarchy(
    virtual_file_system: &VirtualFileSystem<'_>,
    task: TaskIdentifier,
) -> Result<()> {
    virtual_file_system
        .create_directory(&Path::SYSTEM, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::CONFIGURATION, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::SHARED_CONFIGURATION, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::DEVICES, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::USERS, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::DATA, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::SHARED_DATA, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::BINARIES, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::TEMPORARY, task)
        .await?;
    virtual_file_system
        .create_directory(&Path::LOGS, task)
        .await?;

    Ok(())
}

pub async fn clean_devices_in_directory<'a>(
    virtual_file_system: &'a VirtualFileSystem<'a>,
    path: &Path,
) -> Result<()> {
    // For each entry in the directory.
    for entry in Directory::open(virtual_file_system, path).await? {
        if entry.get_type() != Kind::File {
            continue;
        }

        let entry_path = path.append(entry.get_name()).unwrap();

        if virtual_file_system
            .get_metadata_from_path(&entry_path)
            .await?
            .get_type()
            != Kind::CharacterDevice
            && virtual_file_system
                .get_metadata_from_path(&entry_path)
                .await?
                .get_type()
                != Kind::BlockDevice
        {
            continue;
        }

        match virtual_file_system.remove(&entry_path).await {
            Ok(_) | Err(Error::InvalidIdentifier) => {}
            Err(error) => {
                return Err(error);
            }
        }
    }

    Ok(())
}

pub async fn clean_devices<'a>(virtual_file_system: &'a VirtualFileSystem<'a>) -> Result<()> {
    clean_devices_in_directory(virtual_file_system, Path::DEVICES).await?;

    clean_devices_in_directory(virtual_file_system, Path::BINARIES).await?;

    Ok(())
}
