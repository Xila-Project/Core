//! Partition management and device abstraction.
//!
//! This module provides functionality for working with disk partitions, including
//! partition device implementations, partition entry structures, and partition type
//! definitions. It enables treating individual partitions as separate devices while
//! maintaining the underlying device relationships.

mod Device;
mod Entry;
mod Type;

pub use Device::*;
pub use Entry::*;
pub use Type::*;
