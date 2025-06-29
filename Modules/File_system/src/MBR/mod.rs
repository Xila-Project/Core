//! Master Boot Record (MBR) module
//!
//! This module provides functionality for working with MBR (Master Boot Record)
//! partition tables, which are used in traditional BIOS-based systems.
//!
//! # Features
//!
//! - Parse and validate MBR structures from raw bytes or devices
//! - Create and modify MBR partition tables
//! - Work with individual partition entries
//! - Create partition devices for accessing individual partitions
//! - Utility functions for common MBR operations
//! - Type-safe partition type enumeration
//!
//! # Examples
//!
//! ```rust
//! use File_system::MBR::*;
//!
//! // Read MBR from a device
//! let mbr = MBR::Read_from_device(&device)?;
//!
//! // Display MBR information
//! println!("{}", mbr);
//!
//! // Get all valid partitions
//! let partitions = mbr.Get_valid_partitions();
//!
//! // Create a partition device
//! if let Some(partition) = partitions.first() {
//!     let partition_device = Create_partition_device(device.clone(), partition)?;
//! }
//! ```

mod MBR;
mod Utilities;

// Re-export all public items
pub use Utilities::*;
pub use MBR::*;
