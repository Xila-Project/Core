use file_system::{Path_owned_type, Path_type};

use crate::error::{Error_type, Result_type};

pub async fn resolve(command: &str, paths: &[&Path_type]) -> Result_type<Path_owned_type> {
    let virtual_file_system = virtual_file_system::get_instance();
    let task = task::get_instance().get_current_task_identifier().await;

    for path in paths {
        if let Ok(directory) = virtual_file_system.open_directory(path, task).await {
            while let Ok(Some(entry)) = virtual_file_system.read_directory(directory, task).await {
                if entry.get_name() == command {
                    return path.append(command).ok_or(Error_type::Invalid_path);
                }
            }
        }
    }

    Err(Error_type::Command_not_found)
}
