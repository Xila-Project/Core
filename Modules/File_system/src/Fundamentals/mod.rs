//! Fundamental types and structures for file system operations.
//!
//! This module contains the core data types used throughout the file system,
//! including paths, sizes, positions, metadata, permissions, and other essential
//! building blocks for file system operations.

mod Entry;
mod Flags;
mod Identifiers;
mod Path;
mod Position;
mod Size;
mod Statistics;
mod Type;
mod metadata;
mod permission;

pub use metadata::*;
pub use permission::*;
pub use Entry::*;
pub use Flags::*;
pub use Identifiers::*;
pub use Path::*;
pub use Position::*;
pub use Size::*;
pub use Statistics::*;
pub use Type::*;

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
/// use File_system::Block_type;
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
