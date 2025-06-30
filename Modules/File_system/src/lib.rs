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
//! - [`Device_trait`] - Abstraction for storage devices
//! - [`Memory_device_type`] - In-memory device implementation for testing
//! - [`Device_type`] - Thread-safe device wrapper
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
//! - [`Error_type`] - File system error enumeration
//! - [`Size_type`] - Size and position types
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
//! # use File_system::*;
//!
//! // Create an in-memory device for testing
//! let device = Create_device!(Memory_device_type::<512>::New(1024 * 1024));
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
//! # use File_system::*;
//!
//! // Create a device and format it with MBR
//! let device = Create_device!(Memory_device_type::<512>::New(4 * 1024 * 1024));
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
//! All operations return [`Result_type<T>`] which is an alias for `Result<T, Error_type>`.
//! The [`Error_type`] enum provides comprehensive error reporting for all file system operations.

#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod Device;
mod Error;
mod File_system;
mod Fundamentals;
mod MBR;
mod Partition;

mod Memory_device;
mod Time;

pub use Device::{Device_trait, Device_type};
pub use Error::*;

pub use File_system::*;
pub use Fundamentals::*;
pub use Memory_device::*;
pub use Partition::*;
pub use Time::*;

// Export MBR module and its contents
pub use MBR::*;
