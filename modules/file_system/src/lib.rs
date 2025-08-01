//! # File System Module
//!
//! This crate provides a comprehensive file system abstraction layer for the Xila operating system.
//! It includes support for various file system operations, device management, partition handling,
//! and Master Boot Record (MBR) operations.
//!
//! ## Overview
//!
//! The File System module is designed to provide a unified interface for:
//! - File and directory operations (create, read, write, delete, etc.)
//! - Device abstraction for block devices and memory devices
//! - Partition management and MBR (Master Boot Record) support
//! - File system metadata handling (permissions, timestamps, etc.)
//! - Cross-platform file system traits
//!
//! ## Key Components
//!
//! ### File System Traits
//! - [`File_system_traits`] - Core trait for implementing file systems
//! - Support for POSIX-like operations with task and user isolation
//!
//! ### Device Management
//! - [`DeviceTrait`] - Abstraction for storage devices
//! - [`Memory_device_type`] - In-memory device implementation for testing
//! - [`DeviceType`] - Thread-safe device wrapper
//!
//! ### Partition Support
//! - [`Partition_device_type`] - Device representing a partition on a larger device
//! - [`Partition_entry_type`] - MBR partition table entry
//! - [`Partition_type_type`] - Enumeration of partition types
//!
//! ### MBR (Master Boot Record)
//! - [`MBR_type`] - Complete MBR structure with partition table
//! - Utilities for creating, reading, and validating MBRs
//! - Support for creating partition devices from MBR entries
//!
//! ### Fundamental Types
//! - [`Path_type`] - File system path representation
//! - [`Error`] - File system error enumeration
//! - [`Size`] - Size and position types
//! - [`Time_type`] - Timestamp handling
//! - [`Flags_type`] - File operation flags
//!
//! ## Features
//!
//! - `std` - Enables standard library support for environments that have it
//! - Default is `no_std` for embedded systems
//!
//! ## Examples
//!
//! ### Basic Device Operations
//!
//! ```rust
//! # extern crate alloc;
//! # use file_system::*;
//!
//! // Create an in-memory device for testing
//! let device = create_device!(Memory_device_type::<512>::new(1024 * 1024));
//!
//! // Write some data
//! let data = b"Hello, File System!";
//! let result = device.Write(data);
//! assert!(result.is_ok());
//! ```
//!
//! ### MBR Operations
//!
//! ```rust
//! # extern crate alloc;
//! # use file_system::*;
//!
//! // Create a device and format it with MBR
//! let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
//!
//! // Create MBR and add a partition
//! let mut mbr = MBR_type::New_with_signature(0x12345678);
//! mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 8192, true).unwrap();
//!
//! // Write MBR to device
//! mbr.Write_to_device(&device).unwrap();
//!
//! // Create a partition device
//! let partition = Create_partition_device(device, &mbr.Partitions[0]).unwrap();
//! ```
//!
//! ## Safety and Concurrency
//!
//! This crate is designed to be thread-safe and supports concurrent access to file systems
//! and devices. All device implementations must be `Send + Sync` and should use appropriate
//! synchronization primitives to handle concurrent access.
//!
//! ## Error Handling
//!
//! All operations return [`Result<T>`] which is an alias for `Result<T, Error>`.
//! The [`Error`] enum provides comprehensive error reporting for all file system operations.

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod device;
mod error;
mod file_system;
mod fundamentals;
mod mbr;
mod partition;

mod memory_device;
mod time;

pub use device::{Device, DeviceTrait};
pub use error::*;

pub use file_system::*;
pub use fundamentals::*;
pub use memory_device::*;
pub use partition::*;
pub use time::*;

// Export MBR module and its contents
pub use mbr::*;
