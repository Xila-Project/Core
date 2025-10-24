use crate::error::{Error, Result};
use xila::{
    file_system::{Path, PathOwned},
    task, virtual_file_system,
};

pub async fn resolve(command: &str, paths: &[&Path]) -> Result<PathOwned> {
    let virtual_file_system = virtual_file_system::get_instance();
    let task = task::get_instance().get_current_task_identifier().await;

    for path in paths {
        if let Ok(directory) = virtual_file_system.open_directory(path, task).await {
            while let Ok(Some(entry)) = virtual_file_system.read_directory(directory, task).await {
                if entry.get_name() == command {
                    return path.append(command).ok_or(Error::InvalidPath);
                }
            }
        }
    }

    Err(Error::CommandNotFound)
}
