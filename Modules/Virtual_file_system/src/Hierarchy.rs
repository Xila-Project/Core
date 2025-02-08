/// Hierarchy of the file system.
use File_system::{Error_type, Path_type, Result_type, Type_type};
use Task::Task_identifier_type;

use crate::{Directory_type, Virtual_file_system_type};

/// Create the default hierarchy of the file system.
pub fn Create_default_hierarchy(
    Virtual_file_system: &Virtual_file_system_type,
    Task: Task_identifier_type,
) -> Result_type<()> {
    Virtual_file_system.Create_directory(&Path_type::System, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Configuration, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Shared_configuration, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Devices, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Users, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Data, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Shared_data, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Binaries, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Temporary, Task)?;
    Virtual_file_system.Create_directory(&Path_type::Logs, Task)?;

    Ok(())
}

pub fn Clean_devices_in_directory<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Path: &Path_type,
) -> Result_type<()> {
    // For each entry in the directory.
    for Entry in Directory_type::Open(Virtual_file_system, Path)? {
        if Entry.Get_type() != Type_type::File {
            continue;
        }

        let Entry_path = Path.Append(Entry.Get_name()).unwrap();

        let Metadata = Virtual_file_system.Get_metadata_from_path(&Entry_path)?;

        if Metadata.Get_type() != Type_type::Character_device
            && Metadata.Get_type() != Type_type::Block_device
        {
            continue;
        }

        match Virtual_file_system.Remove(&Entry_path) {
            Ok(_) | Err(Error_type::Invalid_identifier) => {
                println!("Removed device: {}", Entry_path);
            }

            Err(Error) => {
                return Err(Error);
            }
        }
    }

    Ok(())
}

pub fn Clean_devices<'a>(Virtual_file_system: &'a Virtual_file_system_type<'a>) -> Result_type<()> {
    Clean_devices_in_directory(Virtual_file_system, Path_type::Devices)?;

    Clean_devices_in_directory(Virtual_file_system, Path_type::Binaries)?;

    Ok(())
}
