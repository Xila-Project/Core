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
/// assert_eq!(file_entry.Get_name(), "document.txt");
/// assert_eq!(file_entry.Get_type(), Type_type::File);
/// assert_eq!(file_entry.Get_size().As_u64(), 1024);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry_type {
    /// The inode number identifying this file system object
    Inode: Inode_type,
    /// The name of this directory entry
    Name: String,
    /// The type of file system object (file, directory, etc.)
    Type: Type_type,
    /// The size of the file system object in bytes
    Size: Size_type,
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
            Inode,
            Name,
            Type,
            Size,
        }
    }

    /// Get the inode number of this directory entry.
    ///
    /// # Returns
    ///
    /// The unique inode number identifying this file system object.
    pub fn Get_inode(&self) -> Inode_type {
        self.Inode
    }

    /// Get the name of this directory entry.
    ///
    /// # Returns
    ///
    /// A reference to the string containing the file or directory name.
    pub fn Get_name(&self) -> &String {
        &self.Name
    }

    /// Get the type of this directory entry.
    ///
    /// # Returns
    ///
    /// The type of file system object (file, directory, symbolic link, etc.).
    pub fn Get_type(&self) -> Type_type {
        self.Type
    }

    /// Get the size of this directory entry.
    ///
    /// # Returns
    ///
    /// For files, this is the size in bytes. For directories, this may represent
    /// the number of entries or be implementation-defined.
    pub fn Get_size(&self) -> Size_type {
        self.Size
    }

    /// Set the inode number for this directory entry.
    ///
    /// # Arguments
    ///
    /// * `Inode` - The new inode number to assign
    pub fn Set_inode(&mut self, Inode: Inode_type) {
        self.Inode = Inode;
    }

    /// Set the name for this directory entry.
    ///
    /// # Arguments
    ///
    /// * `Name` - The new name to assign to this entry
    pub fn Set_name(&mut self, Name: String) {
        self.Name = Name;
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
        self.Size = Size;
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
