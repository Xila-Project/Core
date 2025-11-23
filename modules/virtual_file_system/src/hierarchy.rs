use exported_file_system::Permissions;
/// Hierarchy of the file system.
use file_system::{Kind, Path};
use task::TaskIdentifier;

use crate::{Directory, Error, Result, VirtualFileSystem};

/// Create the default hierarchy of the file system.
pub async fn create_default_hierarchy(
    virtual_file_system: &VirtualFileSystem<'_>,
    task: TaskIdentifier,
) -> Result<()> {
    virtual_file_system
        .set_permissions(task, &Path::ROOT, Permissions::DIRECTORY_DEFAULT)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::SYSTEM)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::CONFIGURATION)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::SHARED_CONFIGURATION)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::DEVICES)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::USERS)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::DATA)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::SHARED_DATA)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::BINARIES)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::TEMPORARY)
        .await?;
    virtual_file_system
        .create_directory(task, &Path::LOGS)
        .await?;

    Ok(())
}

pub async fn clean_devices_in_directory<'a>(
    virtual_file_system: &'a VirtualFileSystem<'a>,
    task: TaskIdentifier,
    path: &Path,
) -> Result<()> {
    // For each entry in the directory.
    for entry in Directory::open(virtual_file_system, task, path).await? {
        if entry.kind != Kind::File {
            continue;
        }

        let entry_path = path.append(&entry.name).unwrap();

        let kind = virtual_file_system.get_statistics(&entry_path).await?.kind;

        if kind != Kind::CharacterDevice && kind != Kind::BlockDevice {
            continue;
        }

        match virtual_file_system.remove(task, &entry_path).await {
            Ok(_) | Err(Error::InvalidIdentifier) => {}
            Err(error) => {
                return Err(error);
            }
        }
    }

    Ok(())
}

pub async fn clean_devices<'a>(
    virtual_file_system: &'a VirtualFileSystem<'a>,
    task: TaskIdentifier,
) -> Result<()> {
    clean_devices_in_directory(virtual_file_system, task, Path::DEVICES).await?;

    clean_devices_in_directory(virtual_file_system, task, Path::BINARIES).await?;

    Ok(())
}
