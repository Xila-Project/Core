//! File system object type definitions.
//!
//! This module provides the [`Type_type`] enumeration which categorizes different
//! types of objects that can exist in a file system, such as regular files,
//! directories, devices, and special file types.

use core::fmt::Display;

/// Enumeration of file system object types.
///
/// This enum represents the different types of objects that can exist in a file system.
/// Each variant corresponds to a specific type of file system entity with its own
/// characteristics and behaviors.
///
/// # Variants
///
/// * [`File`] - Regular file containing data
/// * [`Directory`] - Directory containing other file system objects
/// * [`Block_device`] - Block-oriented device (e.g., disk drives)
/// * [`Character_device`] - Character-oriented device (e.g., terminals, serial ports)
/// * [`Pipe`] - Named pipe (FIFO) for inter-process communication
/// * [`Socket`] - Unix domain socket for inter-process communication
/// * [`Symbolic_link`] - Symbolic link pointing to another file system object
///
/// # Examples
///
/// ```rust
/// use File_system::Type_type;
///
/// let file_type = Type_type::File;
/// let dir_type = Type_type::Directory;
///
/// // Types can be compared
/// assert_ne!(file_type, dir_type);
///
/// // Types can be displayed
/// println!("Object type: {}", file_type); // Prints "Object type: File"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Type_type {
    /// Regular file containing data
    File,
    /// Directory containing other file system objects
    Directory,
    /// Block-oriented device (e.g., disk drives, storage devices)
    Block_device,
    /// Character-oriented device (e.g., terminals, serial ports)
    Character_device,
    /// Named pipe (FIFO) for inter-process communication
    Pipe,
    /// Unix domain socket for inter-process communication
    Socket,
    /// Symbolic link pointing to another file system object
    Symbolic_link,
}

impl Display for Type_type {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let Type = match self {
            Type_type::File => "File",
            Type_type::Directory => "Directory",
            Type_type::Block_device => "Block device",
            Type_type::Character_device => "Character device",
            Type_type::Pipe => "Pipe",
            Type_type::Socket => "Socket",
            Type_type::Symbolic_link => "Symbolic link",
        };

        write!(f, "{Type}")
    }
}
