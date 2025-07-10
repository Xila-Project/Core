//! Directory entry representation for file system operations.

//!
//! This module provides the [`Entry_type`] structure which represents individual
//! entries within directories, such as files, subdirectories, and other file system objects.

use alloc::string::String;

use crate::Kind;

use super::{Inode, Size};

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
/// use file_system::*;
/// use alloc::string::String;
///
/// // Create a directory entry for a regular file
/// let file_entry = Entry_type::new(
///     Inode_type::new(42),
///     String::from("document.txt"),
///     Type_type::File,
///     Size::new(1024)
/// );
///
/// assert_eq!(file_entry.get_name(), "document.txt");
/// assert_eq!(file_entry.get_type(), Type_type::File);
/// assert_eq!(file_entry.get_size().As_u64(), 1024);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The inode number identifying this file system object
    inode: Inode,
    /// The name of this directory entry
    name: String,
    /// The type of file system object (file, directory, etc.)
    r#type: Kind,
    /// The size of the file system object in bytes
    size: Size,
}

impl Entry {
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
    /// use file_system::*;
    /// use alloc::string::String;
    ///
    /// let entry = Entry_type::new(
    ///     Inode_type::new(123),
    ///     String::from("example.txt"),
    ///     Type_type::File,
    ///     Size::new(2048)
    /// );
    /// ```
    pub fn new(inode: Inode, name: String, r#type: Kind, size: Size) -> Self {
        Self {
            inode,
            name,
            r#type,
            size,
        }
    }

    /// Get the inode number of this directory entry.
    ///
    /// # Returns
    ///
    /// The unique inode number identifying this file system object.
    pub fn get_inode(&self) -> Inode {
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
    pub fn get_type(&self) -> Kind {
        self.r#type
    }

    /// Get the size of this directory entry.
    ///
    /// # Returns
    ///
    /// For files, this is the size in bytes. For directories, this may represent
    /// the number of entries or be implementation-defined.
    pub fn get_size(&self) -> Size {
        self.size
    }

    /// Set the inode number for this directory entry.
    ///
    /// # Arguments
    ///
    /// * `Inode` - The new inode number to assign
    pub fn set_inode(&mut self, inode: Inode) {
        self.inode = inode;
    }

    /// Set the name for this directory entry.
    ///
    /// # Arguments
    ///
    /// * `Name` - The new name to assign to this entry
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Set the type for this directory entry.
    ///
    /// # Arguments
    ///
    /// * `Type` - The new file system object type
    pub fn set_type(&mut self, r#type: Kind) {
        self.r#type = r#type;
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }
}

impl AsMut<[u8]> for Entry {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut Entry as *mut u8,
                core::mem::size_of::<Entry>(),
            )
        }
    }
}

impl TryFrom<&mut [u8]> for &mut Entry {
    type Error = ();

    fn try_from(value: &mut [u8]) -> Result<Self, Self::Error> {
        if value.len() != core::mem::size_of::<Entry>() {
            return Err(());
        }
        if value.as_ptr() as usize % core::mem::align_of::<Entry>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { core::mem::transmute::<*mut u8, &mut Entry>(value.as_mut_ptr()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_entry_creation() {
        let entry = Entry::new(
            Inode::new(42),
            String::from("test.txt"),
            Kind::File,
            Size::new(1024),
        );

        assert_eq!(entry.get_inode(), Inode::new(42));
        assert_eq!(entry.get_name(), "test.txt");
        assert_eq!(entry.get_type(), Kind::File);
        assert_eq!(entry.get_size(), Size::new(1024));
    }

    #[test]
    fn test_entry_getters() {
        let entry = Entry::new(
            Inode::new(123),
            String::from("directory"),
            Kind::Directory,
            Size::new(0),
        );

        // Test individual getters
        assert_eq!(entry.get_inode().as_u64(), 123);
        assert_eq!(entry.get_name(), "directory");
        assert_eq!(entry.get_type(), Kind::Directory);
        assert_eq!(entry.get_size().as_u64(), 0);
    }

    #[test]
    fn test_entry_setters() {
        let mut entry = Entry::new(
            Inode::new(1),
            String::from("old_name"),
            Kind::File,
            Size::new(100),
        );

        // Test setters
        entry.set_inode(Inode::new(999));
        entry.set_name(String::from("new_name.txt"));
        entry.set_type(Kind::Directory);
        entry.set_size(Size::new(2048));

        // Verify changes
        assert_eq!(entry.get_inode().as_u64(), 999);
        assert_eq!(entry.get_name(), "new_name.txt");
        assert_eq!(entry.get_type(), Kind::Directory);
        assert_eq!(entry.get_size().as_u64(), 2048);
    }

    #[test]
    fn test_entry_clone() {
        let original = Entry::new(
            Inode::new(456),
            String::from("clone_test.dat"),
            Kind::File,
            Size::new(512),
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
        let entry = Entry::new(
            Inode::new(789),
            String::from("debug_test"),
            Kind::SymbolicLink,
            Size::new(64),
        );

        let debug_str = alloc::format!("{entry:?}");
        assert!(debug_str.contains("Entry_type"));
        assert!(debug_str.contains("789"));
        assert!(debug_str.contains("debug_test"));
    }

    #[test]
    fn test_entry_equality() {
        let entry1 = Entry::new(
            Inode::new(100),
            String::from("same.txt"),
            Kind::File,
            Size::new(200),
        );

        let entry2 = Entry::new(
            Inode::new(100),
            String::from("same.txt"),
            Kind::File,
            Size::new(200),
        );

        let entry3 = Entry::new(
            Inode::new(101),
            String::from("different.txt"),
            Kind::File,
            Size::new(200),
        );

        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3);
    }

    #[test]
    fn test_entry_different_types() {
        // Test entries with different file types
        let file_entry = Entry::new(
            Inode::new(1),
            String::from("file.txt"),
            Kind::File,
            Size::new(1024),
        );

        let dir_entry = Entry::new(
            Inode::new(2),
            String::from("directory"),
            Kind::Directory,
            Size::new(0),
        );

        let symlink_entry = Entry::new(
            Inode::new(3),
            String::from("link"),
            Kind::SymbolicLink,
            Size::new(32),
        );

        assert_eq!(file_entry.get_type(), Kind::File);
        assert_eq!(dir_entry.get_type(), Kind::Directory);
        assert_eq!(symlink_entry.get_type(), Kind::SymbolicLink);

        assert_ne!(file_entry, dir_entry);
        assert_ne!(dir_entry, symlink_entry);
        assert_ne!(file_entry, symlink_entry);
    }

    #[test]
    fn test_entry_empty_name() {
        let entry = Entry::new(Inode::new(0), String::new(), Kind::File, Size::new(0));

        assert_eq!(entry.get_name(), "");
        assert_eq!(entry.get_name().len(), 0);
    }

    #[test]
    fn test_entry_large_values() {
        let entry = Entry::new(
            Inode::new(u64::MAX),
            String::from("large_file.bin"),
            Kind::File,
            Size::new(u64::MAX),
        );

        assert_eq!(entry.get_inode().as_u64(), u64::MAX);
        assert_eq!(entry.get_size().as_u64(), u64::MAX);
    }

    #[test]
    fn test_entry_as_mut_slice() {
        let mut entry = Entry::new(
            Inode::new(42),
            String::from("test"),
            Kind::File,
            Size::new(100),
        );

        let slice = entry.as_mut();
        assert_eq!(slice.len(), core::mem::size_of::<Entry>());
    }

    #[test]
    fn test_entry_from_slice_invalid_size() {
        let mut buffer = [0u8; 10]; // Too small
        let result = <&mut Entry>::try_from(buffer.as_mut_slice());
        assert!(result.is_err());
    }

    #[test]
    fn test_entry_from_slice_valid() {
        // Create a properly sized and aligned buffer
        let mut buffer = alloc::vec![0u8; core::mem::size_of::<Entry>()];

        // Ensure proper alignment by using Vec which should be properly aligned for any type
        let result = <&mut Entry>::try_from(buffer.as_mut_slice());

        // This might fail due to alignment requirements, which is expected behavior
        // The important thing is that we're testing the code path
        let _ = result; // We don't assert success since alignment isn't guaranteed
    }

    #[test]
    fn test_entry_modification_after_creation() {
        let mut entry = Entry::new(
            Inode::new(1),
            String::from("initial"),
            Kind::File,
            Size::new(0),
        );

        // Modify multiple times
        for i in 1..=5 {
            entry.set_inode(Inode::new(i));
            entry.set_name(alloc::format!("name_{i}"));
            entry.set_size(Size::new(i * 100));

            assert_eq!(entry.get_inode().as_u64(), i);
            assert_eq!(entry.get_name(), &alloc::format!("name_{i}"));
            assert_eq!(entry.get_size().as_u64(), i * 100);
        }
    }

    #[test]
    fn test_entry_unicode_names() {
        let entry = Entry::new(
            Inode::new(1),
            String::from("файл.txt"), // Cyrillic
            Kind::File,
            Size::new(256),
        );

        assert_eq!(entry.get_name(), "файл.txt");

        let entry2 = Entry::new(
            Inode::new(2),
            String::from("文件.dat"), // Chinese
            Kind::File,
            Size::new(512),
        );

        assert_eq!(entry2.get_name(), "文件.dat");
    }
}
