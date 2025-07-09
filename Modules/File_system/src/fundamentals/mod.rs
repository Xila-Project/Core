//! Fundamental types and structures for file system operations.
//!
//! This module contains the core data types used throughout the file system,
//! including paths, sizes, positions, metadata, permissions, and other essential
//! building blocks for file system operations.

mod entry;
mod flags;
mod identifiers;
mod metadata;
mod path;
mod permission;
mod position;
mod size;
mod statistics;
mod r#type;

pub use entry::*;
pub use flags::*;
pub use identifiers::*;
pub use metadata::*;
pub use path::*;
pub use permission::*;
pub use position::*;
pub use r#type::*;
pub use size::*;
pub use statistics::*;

/// Standard block size representation for file system operations.
///
/// This type represents a 512-byte block, which is the standard sector size
/// for most storage devices. It's used throughout the file system for
/// block-aligned operations and buffer management.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// use file_system::Block_type;
///
/// let block = Block_type::default();
/// assert_eq!(block.0.len(), 512);
/// ```
#[repr(transparent)]
pub struct Block_type(pub [u8; 512]);

impl Default for Block_type {
    fn default() -> Self {
        Block_type([0; 512])
    }
}
