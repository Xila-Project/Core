use File_system::{Path_owned_type, Path_type};

use crate::Error::{Error_type, Result_type};

pub async fn Resolve(Command: &str, Paths: &[&Path_type]) -> Result_type<Path_owned_type> {
    let virtual_file_system = Virtual_file_system::Get_instance();
    let task = Task::Get_instance().Get_current_task_identifier().await;

    for Path in Paths {
        if let Ok(directory) = virtual_file_system.Open_directory(Path, task).await {
            while let Ok(Some(entry)) = virtual_file_system.Read_directory(directory, task).await {
                if entry.Get_name() == Command {
                    return Path.Append(Command).ok_or(Error_type::Invalid_path);
                }
            }
        }
    }

    Err(Error_type::Command_not_found)
}
