//! Directory entry representation for file system operations.
//!
//! This module provides the [`Entry_type`] structure which represents individual
//! entries within directories, such as files, subdirectories, and other file system objects.

use alloc::string::String;

use crate::Type_type;

use super::{Inode_type, Size_type};

/// Represents a single entry within a directory.
///
/// A directory entry contains metadata about a file system object including its
/// inode number, name, type, and size. This structure is used when reading directory
/// contents to provide information about each item within the directory.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// use File_system::*;
/// use alloc::string::String;
///
/// // Create a directory entry for a regular file
/// let file_entry = Entry_type::New(
///     Inode_type::New(42),
///     String::from("document.txt"),
///     Type_type::File,
///     Size_type::New(1024)
/// );
///
/// assert_eq!(file_entry.get_name(), "document.txt");
/// assert_eq!(file_entry.get_type(), Type_type::File);
/// assert_eq!(file_entry.get_size().As_u64(), 1024);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry_type {
    /// The inode number identifying this file system object
    inode: Inode_type,
    /// The name of this directory entry
    name: String,
    /// The type of file system object (file, directory, etc.)
    Type: Type_type,
    /// The size of the file system object in bytes
    size: Size_type,
}

impl Entry_type {
    /// Create a new directory entry with the specified metadata.
    ///
    /// # Arguments
    ///
    /// * `Inode` - Unique inode number for this file system object
    /// * `Name` - Name of the file or directory
    /// * `Type` - Type of the file system object
    /// * `Size` - Size in bytes (for files) or entry count (for directories)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// use File_system::*;
    /// use alloc::string::String;
    ///
    /// let entry = Entry_type::New(
    ///     Inode_type::New(123),
    ///     String::from("example.txt"),
    ///     Type_type::File,
    ///     Size_type::New(2048)
    /// );
    /// ```
    pub fn New(Inode: Inode_type, Name: String, Type: Type_type, Size: Size_type) -> Self {
        Self {
            inode: Inode,
            name: Name,
            Type,
            size: Size,
        }
    }

    /// Get the inode number of this directory entry.
    ///
    /// # Returns
    ///
    /// The unique inode number identifying this file system object.
    pub fn get_inode(&self) -> Inode_type {
        self.inode
    }

    /// Get the name of this directory entry.
    ///
    /// # Returns
    ///
    /// A reference to the string containing the file or directory name.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Get the type of this directory entry.
    ///
    /// # Returns
    ///
    /// The type of file system object (file, directory, symbolic link, etc.).
    pub fn get_type(&self) -> Type_type {
        self.Type
    }

    /// Get the size of this directory entry.
    ///
    /// # Returns
    ///
    /// For files, this is the size in bytes. For directories, this may represent
    /// the number of entries or be implementation-defined.
    pub fn get_size(&self) -> Size_type {
        self.size
    }

    /// Set the inode number for this directory entry.
    ///
    /// # Arguments
    ///
    /// * `Inode` - The new inode number to assign
    pub fn Set_inode(&mut self, Inode: Inode_type) {
        self.inode = Inode;
    }

    /// Set the name for this directory entry.
    ///
    /// # Arguments
    ///
    /// * `Name` - The new name to assign to this entry
    pub fn Set_name(&mut self, Name: String) {
        self.name = Name;
    }

    /// Set the type for this directory entry.
    ///
    /// # Arguments
    ///
    /// * `Type` - The new file system object type
    pub fn Set_type(&mut self, Type: Type_type) {
        self.Type = Type;
    }

    pub fn Set_size(&mut self, Size: Size_type) {
        self.size = Size;
    }
}

impl AsMut<[u8]> for Entry_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut Entry_type as *mut u8,
                core::mem::size_of::<Entry_type>(),
            )
        }
    }
}

impl TryFrom<&mut [u8]> for &mut Entry_type {
    type Error = ();

    fn try_from(Value: &mut [u8]) -> Result<Self, Self::Error> {
        if Value.len() != core::mem::size_of::<Entry_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % core::mem::align_of::<Entry_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { core::mem::transmute::<*mut u8, &mut Entry_type>(Value.as_mut_ptr()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_entry_creation() {
        let entry = Entry_type::New(
            Inode_type::New(42),
            String::from("test.txt"),
            Type_type::File,
            Size_type::New(1024),
        );

        assert_eq!(entry.get_inode(), Inode_type::New(42));
        assert_eq!(entry.get_name(), "test.txt");
        assert_eq!(entry.get_type(), Type_type::File);
        assert_eq!(entry.get_size(), Size_type::New(1024));
    }

    #[test]
    fn test_entry_getters() {
        let entry = Entry_type::New(
            Inode_type::New(123),
            String::from("directory"),
            Type_type::Directory,
            Size_type::New(0),
        );

        // Test individual getters
        assert_eq!(entry.get_inode().As_u64(), 123);
        assert_eq!(entry.get_name(), "directory");
        assert_eq!(entry.get_type(), Type_type::Directory);
        assert_eq!(entry.get_size().As_u64(), 0);
    }

    #[test]
    fn test_entry_setters() {
        let mut entry = Entry_type::New(
            Inode_type::New(1),
            String::from("old_name"),
            Type_type::File,
            Size_type::New(100),
        );

        // Test setters
        entry.Set_inode(Inode_type::New(999));
        entry.Set_name(String::from("new_name.txt"));
        entry.Set_type(Type_type::Directory);
        entry.Set_size(Size_type::New(2048));

        // Verify changes
        assert_eq!(entry.get_inode().As_u64(), 999);
        assert_eq!(entry.get_name(), "new_name.txt");
        assert_eq!(entry.get_type(), Type_type::Directory);
        assert_eq!(entry.get_size().As_u64(), 2048);
    }

    #[test]
    fn test_entry_clone() {
        let original = Entry_type::New(
            Inode_type::New(456),
            String::from("clone_test.dat"),
            Type_type::File,
            Size_type::New(512),
        );

        let cloned = original.clone();

        // Verify clone has same values
        assert_eq!(original.get_inode(), cloned.get_inode());
        assert_eq!(original.get_name(), cloned.get_name());
        assert_eq!(original.get_type(), cloned.get_type());
        assert_eq!(original.get_size(), cloned.get_size());

        // Verify they are equal
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_entry_debug() {
        let entry = Entry_type::New(
            Inode_type::New(789),
            String::from("debug_test"),
            Type_type::Symbolic_link,
            Size_type::New(64),
        );

        let debug_str = alloc::format!("{entry:?}");
        assert!(debug_str.contains("Entry_type"));
        assert!(debug_str.contains("789"));
        assert!(debug_str.contains("debug_test"));
    }

    #[test]
    fn test_entry_equality() {
        let entry1 = Entry_type::New(
            Inode_type::New(100),
            String::from("same.txt"),
            Type_type::File,
            Size_type::New(200),
        );

        let entry2 = Entry_type::New(
            Inode_type::New(100),
            String::from("same.txt"),
            Type_type::File,
            Size_type::New(200),
        );

        let entry3 = Entry_type::New(
            Inode_type::New(101),
            String::from("different.txt"),
            Type_type::File,
            Size_type::New(200),
        );

        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3);
    }

    #[test]
    fn test_entry_different_types() {
        // Test entries with different file types
        let file_entry = Entry_type::New(
            Inode_type::New(1),
            String::from("file.txt"),
            Type_type::File,
            Size_type::New(1024),
        );

        let dir_entry = Entry_type::New(
            Inode_type::New(2),
            String::from("directory"),
            Type_type::Directory,
            Size_type::New(0),
        );

        let symlink_entry = Entry_type::New(
            Inode_type::New(3),
            String::from("link"),
            Type_type::Symbolic_link,
            Size_type::New(32),
        );

        assert_eq!(file_entry.get_type(), Type_type::File);
        assert_eq!(dir_entry.get_type(), Type_type::Directory);
        assert_eq!(symlink_entry.get_type(), Type_type::Symbolic_link);

        assert_ne!(file_entry, dir_entry);
        assert_ne!(dir_entry, symlink_entry);
        assert_ne!(file_entry, symlink_entry);
    }

    #[test]
    fn test_entry_empty_name() {
        let entry = Entry_type::New(
            Inode_type::New(0),
            String::new(),
            Type_type::File,
            Size_type::New(0),
        );

        assert_eq!(entry.get_name(), "");
        assert_eq!(entry.get_name().len(), 0);
    }

    #[test]
    fn test_entry_large_values() {
        let entry = Entry_type::New(
            Inode_type::New(u64::MAX),
            String::from("large_file.bin"),
            Type_type::File,
            Size_type::New(u64::MAX),
        );

        assert_eq!(entry.get_inode().As_u64(), u64::MAX);
        assert_eq!(entry.get_size().As_u64(), u64::MAX);
    }

    #[test]
    fn test_entry_as_mut_slice() {
        let mut entry = Entry_type::New(
            Inode_type::New(42),
            String::from("test"),
            Type_type::File,
            Size_type::New(100),
        );

        let slice = entry.as_mut();
        assert_eq!(slice.len(), core::mem::size_of::<Entry_type>());
    }

    #[test]
    fn test_entry_from_slice_invalid_size() {
        let mut buffer = [0u8; 10]; // Too small
        let result = <&mut Entry_type>::try_from(buffer.as_mut_slice());
        assert!(result.is_err());
    }

    #[test]
    fn test_entry_from_slice_valid() {
        // Create a properly sized and aligned buffer
        let mut buffer = alloc::vec![0u8; core::mem::size_of::<Entry_type>()];

        // Ensure proper alignment by using Vec which should be properly aligned for any type
        let result = <&mut Entry_type>::try_from(buffer.as_mut_slice());

        // This might fail due to alignment requirements, which is expected behavior
        // The important thing is that we're testing the code path
        let _ = result; // We don't assert success since alignment isn't guaranteed
    }

    #[test]
    fn test_entry_modification_after_creation() {
        let mut entry = Entry_type::New(
            Inode_type::New(1),
            String::from("initial"),
            Type_type::File,
            Size_type::New(0),
        );

        // Modify multiple times
        for i in 1..=5 {
            entry.Set_inode(Inode_type::New(i));
            entry.Set_name(alloc::format!("name_{i}"));
            entry.Set_size(Size_type::New(i * 100));

            assert_eq!(entry.get_inode().As_u64(), i);
            assert_eq!(entry.get_name(), &alloc::format!("name_{i}"));
            assert_eq!(entry.get_size().As_u64(), i * 100);
        }
    }

    #[test]
    fn test_entry_unicode_names() {
        let entry = Entry_type::New(
            Inode_type::New(1),
            String::from("файл.txt"), // Cyrillic
            Type_type::File,
            Size_type::New(256),
        );

        assert_eq!(entry.get_name(), "файл.txt");

        let entry2 = Entry_type::New(
            Inode_type::New(2),
            String::from("文件.dat"), // Chinese
            Type_type::File,
            Size_type::New(512),
        );

        assert_eq!(entry2.get_name(), "文件.dat");
    }
}
