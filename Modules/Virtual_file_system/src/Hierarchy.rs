/// Hierarchy of the file system.
use File_system::{Path_type, Result_type};
use Task::Task_identifier_type;

use crate::Virtual_file_system_type;

/// Contains the OS core, including the kernel, init system, and critical drivers.
/// Prevents modification by regular users.
pub const Xila_directory: &Path_type = Path_type::From_str("/Xila");

/// Stores system-wide settings in a structured format (e.g., JSON, TOML).
pub const Configuration_directory: &Path_type = Path_type::From_str("/Configuration");

/// Hardware devices, symlinks for human-friendly names.
pub const Devices_directory: &Path_type = Path_type::From_str("/Devices");

/// Contains the system's binaries, including the shell and other executables.
pub const Binaries_directory: &Path_type = Path_type::From_str("/Binaries");

/// Contains the user's data, including documents, downloads, and other files.
pub const Users_directory: &Path_type = Path_type::From_str("/Users");

/// Contains temporary files, including logs and caches.
pub const Temporary_directory: &Path_type = Path_type::From_str("/Temporary");

/// Contains logs, including system logs and application logs.
pub const Logs_directory: &Path_type = Path_type::From_str("/Temporary/Logs");

/// Create the default hierarchy of the file system.
pub fn Create_default_hierarchy(
    Virtual_file_system: &Virtual_file_system_type,
    Task: Task_identifier_type,
) -> Result_type<()> {
    Virtual_file_system.Create_directory(&Xila_directory, Task)?;
    Virtual_file_system.Create_directory(&Configuration_directory, Task)?;
    Virtual_file_system.Create_directory(&Devices_directory, Task)?;
    Virtual_file_system.Create_directory(&Binaries_directory, Task)?;
    Virtual_file_system.Create_directory(&Users_directory, Task)?;
    Virtual_file_system.Create_directory(&Temporary_directory, Task)?;
    Virtual_file_system.Create_directory(&Logs_directory, Task)?;

    Ok(())
}
