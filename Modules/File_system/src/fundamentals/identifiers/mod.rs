//! Identifier types for file system objects and operations.
//!
//! This module provides various identifier types used throughout the file system
//! to uniquely identify files, directories, file systems, inodes, and other objects.
//! These identifiers ensure type safety and provide a consistent way to reference
//! file system entities across different operations.

mod entry;
mod file;
mod file_system;
mod inode;
mod local_file;
mod unique_file;

pub use entry::*;
pub use file::*;
pub use file_system::*;
pub use inode::*;
pub use local_file::*;
pub use unique_file::*;
