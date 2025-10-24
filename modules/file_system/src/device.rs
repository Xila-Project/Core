//! Device abstraction for storage and I/O operations.
//!
//! This module provides the core device trait and types for abstracting various
//! storage devices, peripherals, and I/O endpoints in the file system.

use core::fmt;
use core::fmt::{Debug, Formatter};

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use crate::{Error, Position, Result, Size};

/// Convenience macro for creating a new [`Device`] from any type implementing [`DeviceTrait`].
///
/// This macro wraps the provided device implementation in an `Arc` and creates a `Device`.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use file_system::*;
///
/// let memory_device = MemoryDevice::<512>::new(1024);
/// let device = create_device!(memory_device);
/// ```
#[macro_export]
macro_rules! create_device {
    ($Device:expr) => {
        $crate::Device::new(alloc::sync::Arc::new($Device))
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
/// return [`Error::RessourceBusy`] instead. This means implementations should prefer
/// `try_read()` and `try_write()` variants of synchronization primitives.
///
/// ## Position Management
///
/// Devices maintain an internal position cursor that affects read and write operations.
/// The position can be manipulated using [`DeviceTrait::set_position`].
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use file_system::*;
///
/// // Create a memory device for testing
/// let device = create_device!(MemoryDevice::<512>::new(1024));
///
/// // Write data
/// let data = b"Hello, World!";
/// let bytes_written = device.write(data).unwrap();
/// assert_eq!(bytes_written.as_u64(), data.len() as u64);
///
/// // Reset position and read back
/// device.set_position(&Position::Start(0)).unwrap();
/// let mut buffer = alloc::vec![0u8; data.len()];
/// let bytes_read = device.read(&mut buffer).unwrap();
/// assert_eq!(bytes_read.as_u64(), data.len() as u64);
/// assert_eq!(&buffer, data);
/// ```
pub trait DeviceTrait: Send + Sync {
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
    /// * `Ok(Size)` - Number of bytes successfully read
    /// * `Err(Error)` - Error if read operation failed
    ///
    /// # Errors
    ///
    /// * [`Error::InputOutput`] - I/O error during read operation
    /// * [`Error::RessourceBusy`] - Device is temporarily unavailable
    /// * [`Error::InvalidParameter`] - Invalid buffer or device state
    fn read(&self, buffer: &mut [u8]) -> Result<Size>;

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
    /// * `Ok(Size)` - Number of bytes successfully written
    /// * `Err(Error)` - Error if write operation failed
    ///
    /// # Errors
    ///
    /// * [`Error::InputOutput`] - I/O error during write operation
    /// * [`Error::NoSpaceLeft`] - Device is full
    /// * [`Error::RessourceBusy`] - Device is temporarily unavailable
    /// * [`Error::PermissionDenied`] - Device is read-only
    fn write(&self, buffer: &[u8]) -> Result<Size>;

    /// Get the total size of the device in bytes.
    ///
    /// Returns the maximum amount of data that can be stored on or read from the device.
    ///
    /// # Returns
    ///
    /// * `Ok(Size)` - Total device size in bytes
    /// * `Err(Error)` - Error if size cannot be determined
    fn get_size(&self) -> Result<Size>;

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
    /// * `Ok(Size)` - The new absolute position after the operation
    /// * `Err(Error)` - Error if position is invalid
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidParameter`] - Position is beyond device bounds
    fn set_position(&self, position: &Position) -> Result<Size>;

    /// Flush any buffered data to the underlying storage.
    ///
    /// Ensures that all pending write operations are committed to the physical device.
    /// This is important for data integrity, especially on buffered devices.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Flush completed successfully
    /// * `Err(Error)` - Error during flush operation
    fn flush(&self) -> Result<()>;

    /// Erase the entire device.
    ///
    /// This operation is primarily intended for flash memory devices and similar
    /// storage that requires explicit erase operations. For most devices, this
    /// operation is not supported and will return [`Error::UnsupportedOperation`].
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Erase completed successfully
    /// * `Err(Error::UnsupportedOperation)` - Device doesn't support erase
    /// * `Err(Error)` - Error during erase operation
    fn erase(&self) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    /// Get the block size of the device in bytes.
    ///
    /// For block devices, this returns the minimum unit of data transfer.
    /// Operations should ideally be aligned to block boundaries for optimal performance.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Block size in bytes
    /// * `Err(Error::UnsupportedOperation)` - Device doesn't have a block size
    fn get_block_size(&self) -> Result<usize> {
        Err(Error::UnsupportedOperation)
    }

    /// Check if this device represents a terminal/console device.
    ///
    /// # Returns
    ///
    /// * `true` - Device is a terminal
    /// * `false` - Device is not a terminal
    fn is_a_terminal(&self) -> bool {
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
    fn is_a_block_device(&self) -> bool {
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
    /// * `Err(Error)` - Error reading device
    ///
    /// # Note
    ///
    /// This operation can consume significant memory for large devices.
    /// Use with caution on production systems.
    fn dump_device(&self) -> Result<Vec<u8>> {
        let size = self.get_size()?;

        let mut buffer = vec![0; size.into()];

        self.read(&mut buffer)?;

        Ok(buffer)
    }
}

/// Thread-safe wrapper for device implementations.
///
/// `Device` provides a unified interface for all device implementations by wrapping
/// them in an `Arc<dyn DeviceTrait>`. This allows for efficient cloning and sharing of
/// device references across threads while maintaining type erasure.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use file_system::*;
/// # use alloc::sync::Arc;
///
/// // Create a device using the convenience macro
/// let device1 = create_device!(MemoryDevice::<512>::new(1024));
///
/// // Create a device manually
/// let memory_device = MemoryDevice::<512>::new(1024);
/// let device2 = Device::new(Arc::new(memory_device));
///
/// // Clone the device (cheap operation - only clones the Arc)
/// let device_clone = device1.clone();
/// ```
#[derive(Clone)]
#[repr(transparent)]
pub struct Device(Arc<dyn DeviceTrait>);

impl Debug for Device {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Device")
    }
}

impl Device {
    /// Create a new device wrapper from any implementation of [`DeviceTrait`].
    ///
    /// # Arguments
    ///
    /// * `Device` - Arc containing the device implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::*;
    /// # use alloc::sync::Arc;
    ///
    /// let memory_device = MemoryDevice::<512>::new(1024);
    /// let device = Device::new(Arc::new(memory_device));
    /// ```
    pub fn new(device: Arc<dyn DeviceTrait>) -> Self {
        Device(device)
    }

    /// Read data from the device at the current position.
    ///
    /// See [`DeviceTrait::read`] for detailed documentation.
    pub fn read(&self, buffer: &mut [u8]) -> Result<Size> {
        self.0.read(buffer)
    }

    /// Write data to the device at the current position.
    ///
    /// See [`DeviceTrait::write`] for detailed documentation.
    pub fn write(&self, buffer: &[u8]) -> Result<Size> {
        self.0.write(buffer)
    }

    /// Get the total size of the device in bytes.
    ///
    /// See [`DeviceTrait::get_size`] for detailed documentation.
    pub fn get_size(&self) -> Result<Size> {
        self.0.get_size()
    }

    /// Set the current position cursor for read/write operations.
    ///
    /// See [`DeviceTrait::set_position`] for detailed documentation.
    pub fn set_position(&self, position: &Position) -> Result<Size> {
        self.0.set_position(position)
    }

    /// Flush any buffered data to the underlying storage.
    ///
    /// See [`DeviceTrait::flush`] for detailed documentation.
    pub fn flush(&self) -> Result<()> {
        self.0.flush()
    }

    /// Erase the entire device.
    ///
    /// See [`DeviceTrait::erase`] for detailed documentation.
    pub fn erase(&self) -> Result<()> {
        self.0.erase()
    }

    /// Get the block size of the device in bytes.
    ///
    /// See [`DeviceTrait::get_block_size`] for detailed documentation.
    pub fn get_block_size(&self) -> Result<usize> {
        self.0.get_block_size()
    }

    /// Check if this device represents a terminal/console device.
    ///
    /// See [`DeviceTrait::is_a_terminal`] for detailed documentation.
    pub fn is_a_terminal(&self) -> bool {
        self.0.is_a_terminal()
    }

    /// Check if this device is a block device.
    ///
    /// See [`DeviceTrait::is_a_block_device`] for detailed documentation.
    pub fn is_a_block_device(&self) -> bool {
        self.0.is_a_block_device()
    }

    /// Create a complete dump of the device contents.
    ///
    /// See [`DeviceTrait::dump_device`] for detailed documentation.
    pub fn dump_device(&self) -> Result<Vec<u8>> {
        self.0.dump_device()
    }
}
