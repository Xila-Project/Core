//! Device abstraction for storage and I/O operations.
//!
//! This module provides the core device trait and types for abstracting various
//! storage devices, peripherals, and I/O endpoints in the file system.

use core::fmt;
use core::fmt::{Debug, Formatter};

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use crate::{Error_type, Position_type, Result_type, Size_type};

/// Convenience macro for creating a new [`Device_type`] from any type implementing [`Device_trait`].
///
/// This macro wraps the provided device implementation in an `Arc` and creates a `Device_type`.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use File_system::*;
///
/// let memory_device = Memory_device_type::<512>::New(1024);
/// let device = Create_device!(memory_device);
/// ```
#[macro_export]
macro_rules! Create_device {
    ($Device:expr) => {
        $crate::Device_type::New(alloc::sync::Arc::new($Device))
    };
}

/// Core trait for all device implementations in the file system.
///
/// A device represents any storage medium or I/O endpoint that can be read from and written to.
/// This includes physical storage devices (hard drives, SSDs, SD cards), memory devices for testing,
/// partition devices, and other specialized I/O devices.
///
/// ## Thread Safety
///
/// All device implementations must be thread-safe (`Send + Sync`) as they may be accessed
/// by multiple tasks/threads concurrently. Implementations should use appropriate synchronization
/// primitives like `RwLock` or `Mutex` to handle concurrent access.
///
/// ## Non-Blocking Operations
///
/// Devices should never block indefinitely. If an operation would block, implementations should
/// return [`Error_type::Ressource_busy`] instead. This means implementations should prefer
/// `try_read()` and `try_write()` variants of synchronization primitives.
///
/// ## Position Management
///
/// Devices maintain an internal position cursor that affects read and write operations.
/// The position can be manipulated using [`Device_trait::Set_position`].
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use File_system::*;
///
/// // Create a memory device for testing
/// let device = Create_device!(Memory_device_type::<512>::New(1024));
///
/// // Write data
/// let data = b"Hello, World!";
/// let bytes_written = device.Write(data).unwrap();
/// assert_eq!(bytes_written.As_u64(), data.len() as u64);
///
/// // Reset position and read back
/// device.Set_position(&Position_type::Start(0)).unwrap();
/// let mut buffer = alloc::vec![0u8; data.len()];
/// let bytes_read = device.Read(&mut buffer).unwrap();
/// assert_eq!(bytes_read.As_u64(), data.len() as u64);
/// assert_eq!(&buffer, data);
/// ```
pub trait Device_trait: Send + Sync {
    /// Read data from the device at the current position.
    ///
    /// Reads up to `Buffer.len()` bytes from the device into the provided buffer.
    /// The actual number of bytes read may be less than requested.
    ///
    /// # Arguments
    ///
    /// * `Buffer` - Mutable byte slice to read data into
    ///
    /// # Returns
    ///
    /// * `Ok(Size_type)` - Number of bytes successfully read
    /// * `Err(Error_type)` - Error if read operation failed
    ///
    /// # Errors
    ///
    /// * [`Error_type::Input_output`] - I/O error during read operation
    /// * [`Error_type::Ressource_busy`] - Device is temporarily unavailable
    /// * [`Error_type::Invalid_parameter`] - Invalid buffer or device state
    fn Read(&self, Buffer: &mut [u8]) -> Result_type<Size_type>;

    /// Write data to the device at the current position.
    ///
    /// Writes up to `Buffer.len()` bytes from the buffer to the device.
    /// The actual number of bytes written may be less than requested.
    ///
    /// # Arguments
    ///
    /// * `Buffer` - Byte slice containing data to write
    ///
    /// # Returns
    ///
    /// * `Ok(Size_type)` - Number of bytes successfully written
    /// * `Err(Error_type)` - Error if write operation failed
    ///
    /// # Errors
    ///
    /// * [`Error_type::Input_output`] - I/O error during write operation
    /// * [`Error_type::No_space_left`] - Device is full
    /// * [`Error_type::Ressource_busy`] - Device is temporarily unavailable
    /// * [`Error_type::Permission_denied`] - Device is read-only
    fn Write(&self, Buffer: &[u8]) -> Result_type<Size_type>;

    /// Get the total size of the device in bytes.
    ///
    /// Returns the maximum amount of data that can be stored on or read from the device.
    ///
    /// # Returns
    ///
    /// * `Ok(Size_type)` - Total device size in bytes
    /// * `Err(Error_type)` - Error if size cannot be determined
    fn Get_size(&self) -> Result_type<Size_type>;

    /// Set the current position cursor for read/write operations.
    ///
    /// The position affects where subsequent read and write operations will occur.
    /// Different position types allow for absolute positioning, relative positioning,
    /// and positioning from the end of the device.
    ///
    /// # Arguments
    ///
    /// * `Position` - The new position to set
    ///
    /// # Returns
    ///
    /// * `Ok(Size_type)` - The new absolute position after the operation
    /// * `Err(Error_type)` - Error if position is invalid
    ///
    /// # Errors
    ///
    /// * [`Error_type::Invalid_parameter`] - Position is beyond device bounds
    fn Set_position(&self, Position: &Position_type) -> Result_type<Size_type>;

    /// Flush any buffered data to the underlying storage.
    ///
    /// Ensures that all pending write operations are committed to the physical device.
    /// This is important for data integrity, especially on buffered devices.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Flush completed successfully
    /// * `Err(Error_type)` - Error during flush operation
    fn Flush(&self) -> Result_type<()>;

    /// Erase the entire device.
    ///
    /// This operation is primarily intended for flash memory devices and similar
    /// storage that requires explicit erase operations. For most devices, this
    /// operation is not supported and will return [`Error_type::Unsupported_operation`].
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Erase completed successfully
    /// * `Err(Error_type::Unsupported_operation)` - Device doesn't support erase
    /// * `Err(Error_type)` - Error during erase operation
    fn Erase(&self) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    /// Get the block size of the device in bytes.
    ///
    /// For block devices, this returns the minimum unit of data transfer.
    /// Operations should ideally be aligned to block boundaries for optimal performance.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Block size in bytes
    /// * `Err(Error_type::Unsupported_operation)` - Device doesn't have a block size
    fn Get_block_size(&self) -> Result_type<usize> {
        Err(Error_type::Unsupported_operation)
    }

    /// Check if this device represents a terminal/console device.
    ///
    /// # Returns
    ///
    /// * `true` - Device is a terminal
    /// * `false` - Device is not a terminal
    fn Is_a_terminal(&self) -> bool {
        false
    }

    /// Check if this device is a block device.
    ///
    /// Block devices support fixed-size block operations and typically represent
    /// physical storage media like hard drives or SSDs.
    ///
    /// # Returns
    ///
    /// * `true` - Device is a block device
    /// * `false` - Device is not a block device
    fn Is_a_block_device(&self) -> bool {
        false
    }

    /// Create a complete dump of the device contents.
    ///
    /// Reads the entire device contents into a vector. This is primarily useful
    /// for debugging, testing, and creating backups of small devices.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Complete device contents
    /// * `Err(Error_type)` - Error reading device
    ///
    /// # Note
    ///
    /// This operation can consume significant memory for large devices.
    /// Use with caution on production systems.
    fn Dump_device(&self) -> Result_type<Vec<u8>> {
        let size = self.Get_size()?;

        let mut Buffer = vec![0; size.into()];

        self.Read(&mut Buffer)?;

        Ok(Buffer)
    }
}

/// Thread-safe wrapper for device implementations.
///
/// `Device_type` provides a unified interface for all device implementations by wrapping
/// them in an `Arc<dyn Device_trait>`. This allows for efficient cloning and sharing of
/// device references across threads while maintaining type erasure.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use File_system::*;
/// # use alloc::sync::Arc;
///
/// // Create a device using the convenience macro
/// let device1 = Create_device!(Memory_device_type::<512>::New(1024));
///
/// // Create a device manually
/// let memory_device = Memory_device_type::<512>::New(1024);
/// let device2 = Device_type::New(Arc::new(memory_device));
///
/// // Clone the device (cheap operation - only clones the Arc)
/// let device_clone = device1.clone();
/// ```
#[derive(Clone)]
#[repr(transparent)]
pub struct Device_type(Arc<dyn Device_trait>);

impl Debug for Device_type {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Device_type")
    }
}

impl Device_type {
    /// Create a new device wrapper from any implementation of [`Device_trait`].
    ///
    /// # Arguments
    ///
    /// * `Device` - Arc containing the device implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use File_system::*;
    /// # use alloc::sync::Arc;
    ///
    /// let memory_device = Memory_device_type::<512>::New(1024);
    /// let device = Device_type::New(Arc::new(memory_device));
    /// ```
    pub fn New(Device: Arc<dyn Device_trait>) -> Self {
        Device_type(Device)
    }

    /// Read data from the device at the current position.
    ///
    /// See [`Device_trait::Read`] for detailed documentation.
    pub fn Read(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        self.0.Read(Buffer)
    }

    /// Write data to the device at the current position.
    ///
    /// See [`Device_trait::Write`] for detailed documentation.
    pub fn Write(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        self.0.Write(Buffer)
    }

    /// Get the total size of the device in bytes.
    ///
    /// See [`Device_trait::Get_size`] for detailed documentation.
    pub fn Get_size(&self) -> Result_type<Size_type> {
        self.0.Get_size()
    }

    /// Set the current position cursor for read/write operations.
    ///
    /// See [`Device_trait::Set_position`] for detailed documentation.
    pub fn Set_position(&self, Position: &Position_type) -> Result_type<Size_type> {
        self.0.Set_position(Position)
    }

    /// Flush any buffered data to the underlying storage.
    ///
    /// See [`Device_trait::Flush`] for detailed documentation.
    pub fn Flush(&self) -> Result_type<()> {
        self.0.Flush()
    }

    /// Erase the entire device.
    ///
    /// See [`Device_trait::Erase`] for detailed documentation.
    pub fn Erase(&self) -> Result_type<()> {
        self.0.Erase()
    }

    /// Get the block size of the device in bytes.
    ///
    /// See [`Device_trait::Get_block_size`] for detailed documentation.
    pub fn Get_block_size(&self) -> Result_type<usize> {
        self.0.Get_block_size()
    }

    /// Check if this device represents a terminal/console device.
    ///
    /// See [`Device_trait::Is_a_terminal`] for detailed documentation.
    pub fn Is_a_terminal(&self) -> bool {
        self.0.Is_a_terminal()
    }

    /// Check if this device is a block device.
    ///
    /// See [`Device_trait::Is_a_block_device`] for detailed documentation.
    pub fn Is_a_block_device(&self) -> bool {
        self.0.Is_a_block_device()
    }

    /// Create a complete dump of the device contents.
    ///
    /// See [`Device_trait::Dump_device`] for detailed documentation.
    pub fn Dump_device(&self) -> Result_type<Vec<u8>> {
        self.0.Dump_device()
    }
}
