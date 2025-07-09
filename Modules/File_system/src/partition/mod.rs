//! Partition management and device abstraction.
//!
//! This module provides functionality for working with disk partitions, including
//! partition device implementations, partition entry structures, and partition type
//! definitions. It enables treating individual partitions as separate devices while
//! maintaining the underlying device relationships.

mod device;
mod entry;
mod statistics;
mod r#type;

pub use device::*;
pub use entry::*;
pub use r#type::*;
pub use statistics::*;
