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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[cfg(test)]
mod Tests {
    use super::*;
    use alloc::format;

    #[test]
    fn Test_type_variants() {
        // Test that all variants can be created
        let file = Type_type::File;
        let directory = Type_type::Directory;
        let block_device = Type_type::Block_device;
        let character_device = Type_type::Character_device;
        let pipe = Type_type::Pipe;
        let socket = Type_type::Socket;
        let symbolic_link = Type_type::Symbolic_link;

        // Basic verification they're different
        assert_ne!(file, directory);
        assert_ne!(directory, block_device);
        assert_ne!(block_device, character_device);
        assert_ne!(character_device, pipe);
        assert_ne!(pipe, socket);
        assert_ne!(socket, symbolic_link);
    }

    #[test]
    fn Test_type_equality() {
        let file1 = Type_type::File;
        let file2 = Type_type::File;
        let directory = Type_type::Directory;

        assert_eq!(file1, file2);
        assert_ne!(file1, directory);
    }

    #[test]
    fn Test_type_clone_copy() {
        let original = Type_type::File;
        let cloned = original.clone();
        let copied = original;

        assert_eq!(original, cloned);
        assert_eq!(original, copied);
        assert_eq!(cloned, copied);
    }

    #[test]
    fn Test_type_debug() {
        let file_type = Type_type::File;
        let debug_str = format!("{:?}", file_type);
        assert_eq!(debug_str, "File");

        let directory_type = Type_type::Directory;
        let debug_str = format!("{:?}", directory_type);
        assert_eq!(debug_str, "Directory");
    }

    #[test]
    fn Test_type_display() {
        assert_eq!(format!("{}", Type_type::File), "File");
        assert_eq!(format!("{}", Type_type::Directory), "Directory");
        assert_eq!(format!("{}", Type_type::Block_device), "Block device");
        assert_eq!(
            format!("{}", Type_type::Character_device),
            "Character device"
        );
        assert_eq!(format!("{}", Type_type::Pipe), "Pipe");
        assert_eq!(format!("{}", Type_type::Socket), "Socket");
        assert_eq!(format!("{}", Type_type::Symbolic_link), "Symbolic link");
    }

    #[test]
    fn Test_type_repr() {
        // Test that the enum has a specific memory representation
        use core::mem::size_of;

        // Should be 1 byte due to repr(u8)
        assert_eq!(size_of::<Type_type>(), 1);
    }

    #[test]
    fn Test_type_discriminants() {
        // Test that different variants have different discriminants
        let file = Type_type::File as u8;
        let directory = Type_type::Directory as u8;
        let block_device = Type_type::Block_device as u8;
        let character_device = Type_type::Character_device as u8;
        let pipe = Type_type::Pipe as u8;
        let socket = Type_type::Socket as u8;
        let symbolic_link = Type_type::Symbolic_link as u8;

        // All should be different
        let discriminants = [
            file,
            directory,
            block_device,
            character_device,
            pipe,
            socket,
            symbolic_link,
        ];

        for i in 0..discriminants.len() {
            for j in i + 1..discriminants.len() {
                assert_ne!(
                    discriminants[i], discriminants[j],
                    "Discriminants {} and {} should be different",
                    i, j
                );
            }
        }
    }

    #[test]
    fn Test_type_pattern_matching() {
        let file_type = Type_type::File;

        let description = match file_type {
            Type_type::File => "regular file",
            Type_type::Directory => "directory",
            Type_type::Block_device => "block device",
            Type_type::Character_device => "character device",
            Type_type::Pipe => "pipe",
            Type_type::Socket => "socket",
            Type_type::Symbolic_link => "symbolic link",
        };

        assert_eq!(description, "regular file");
    }

    #[test]
    fn Test_type_all_variants_pattern_matching() {
        let types = [
            Type_type::File,
            Type_type::Directory,
            Type_type::Block_device,
            Type_type::Character_device,
            Type_type::Pipe,
            Type_type::Socket,
            Type_type::Symbolic_link,
        ];

        for type_variant in types.iter() {
            // Test that each variant can be matched
            let _matched = match type_variant {
                Type_type::File => "file",
                Type_type::Directory => "directory",
                Type_type::Block_device => "block",
                Type_type::Character_device => "char",
                Type_type::Pipe => "pipe",
                Type_type::Socket => "socket",
                Type_type::Symbolic_link => "symlink",
            };
        }
    }

    #[test]
    fn Test_type_in_collections() {
        use alloc::vec::Vec;

        let mut types = Vec::new();
        types.push(Type_type::File);
        types.push(Type_type::Directory);
        types.push(Type_type::Symbolic_link);

        assert_eq!(types.len(), 3);
        assert_eq!(types[0], Type_type::File);
        assert_eq!(types[1], Type_type::Directory);
        assert_eq!(types[2], Type_type::Symbolic_link);
    }

    #[test]
    fn Test_type_is_specific_type() {
        // Helper functions that might be useful
        fn is_file(t: Type_type) -> bool {
            matches!(t, Type_type::File)
        }

        fn is_directory(t: Type_type) -> bool {
            matches!(t, Type_type::Directory)
        }

        fn is_device(t: Type_type) -> bool {
            matches!(t, Type_type::Block_device | Type_type::Character_device)
        }

        assert!(is_file(Type_type::File));
        assert!(!is_file(Type_type::Directory));

        assert!(is_directory(Type_type::Directory));
        assert!(!is_directory(Type_type::File));

        assert!(is_device(Type_type::Block_device));
        assert!(is_device(Type_type::Character_device));
        assert!(!is_device(Type_type::File));
    }

    #[test]
    fn Test_type_default_behavior() {
        // Test that we can use Type_type in various contexts
        let mut type_counts = alloc::collections::BTreeMap::new();

        let types = [
            Type_type::File,
            Type_type::File,
            Type_type::Directory,
            Type_type::File,
            Type_type::Symbolic_link,
        ];

        for t in types.iter() {
            *type_counts.entry(*t).or_insert(0) += 1;
        }

        assert_eq!(type_counts[&Type_type::File], 3);
        assert_eq!(type_counts[&Type_type::Directory], 1);
        assert_eq!(type_counts[&Type_type::Symbolic_link], 1);
    }
}
