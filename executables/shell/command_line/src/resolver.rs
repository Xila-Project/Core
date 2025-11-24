use crate::error::{Error, Result};
use xila::{
    file_system::{Path, PathOwned},
    task,
    virtual_file_system::{self, Directory},
};

pub async fn resolve(command: &str, paths: &[&Path]) -> Result<PathOwned> {
    let virtual_file_system = virtual_file_system::get_instance();
    let task = task::get_instance().get_current_task_identifier().await;

    for path in paths {
        if let Ok(mut directory) = Directory::open(virtual_file_system, task, path).await {
            while let Ok(Some(entry)) = directory.read().await {
                if entry.name == command {
                    return path.append(command).ok_or(Error::InvalidPath);
                }
            }
        }
    }

    Err(Error::CommandNotFound)
}
