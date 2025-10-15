//! In-memory device implementation for testing and simulation.

//!
//! This module provides a memory-based device implementation that stores data
//! in RAM instead of on physical storage. It's primarily used for testing,
//! simulation, and development purposes where you need a device that behaves
//! like storage but doesn't require actual hardware.

use alloc::vec;
use alloc::vec::Vec;
use futures::block_on;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use crate::{DeviceTrait, Position, Size};

/// In-memory device implementation with configurable block size.
///
/// This device stores all data in memory using a `Vec<u8>` and provides the same
/// interface as physical storage devices. It's thread-safe and supports all standard
/// device operations. The block size is configurable at compile time through the
/// const generic parameter.
///
/// # Type Parameters
///
/// * `Block_size` - The block size in bytes (must be a power of 2, typically 512)
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use file_system::*;
///
/// // Create a 1MB memory device with 512-byte blocks
/// let device = Memory_device_type::<512>::new(1024 * 1024);
/// let device = create_device!(device);
///
/// // Write some data
/// let data = b"Hello, Memory Device!";
/// device.Write(data).unwrap();
///
/// // Reset position and read back
/// device.Set_position(&Position_type::Start(0)).unwrap();
/// let mut buffer = alloc::vec![0u8; data.len()];
/// device.Read(&mut buffer).unwrap();
/// assert_eq!(&buffer, data);
/// ```
///
/// # Thread Safety
///
/// The device uses an `RwLock` to ensure thread-safe access to the underlying data.
/// Multiple readers can access the device simultaneously, but writes are exclusive.
pub struct MemoryDevice<const BLOCK_SIZE: usize>(RwLock<CriticalSectionRawMutex, (Vec<u8>, usize)>);

impl<const BLOCK_SIZE: usize> MemoryDevice<BLOCK_SIZE> {
    /// Create a new memory device with the specified size.
    ///
    /// The device will be initialized with zeros and have the specified total size.
    /// The size must be a multiple of the block size.
    ///
    /// # Arguments
    ///
    /// * `Size` - Total size of the device in bytes
    ///
    /// # Panics
    ///
    /// Panics if `Size` is not a multiple of `Block_size`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::Memory_device_type;
    ///
    /// // Create a 4KB device with 512-byte blocks
    /// let device = Memory_device_type::<512>::new(4096);
    /// ```
    pub fn new(size: usize) -> Self {
        assert!(size.is_multiple_of(BLOCK_SIZE));

        let data: Vec<u8> = vec![0; size];

        Self(RwLock::new((data, 0)))
    }

    /// Create a memory device from existing data.
    ///
    /// This allows you to create a device with pre-populated data, useful for
    /// testing with known data patterns or loading device images.
    ///
    /// # Arguments
    ///
    /// * `Data` - Vector containing the initial device data
    ///
    /// # Panics
    ///
    /// Panics if the data length is not a multiple of `Block_size`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::Memory_device_type;
    /// # use alloc::vec;
    ///
    /// // Create device with specific data
    /// let data = vec![0x42; 1024]; // 1KB of 0x42 bytes
    /// let device = Memory_device_type::<512>::From_vec(data);
    /// ```
    pub fn from_vec(data: Vec<u8>) -> Self {
        assert!(data.len().is_multiple_of(BLOCK_SIZE));

        Self(RwLock::new((data, 0)))
    }

    /// Get the number of blocks in this device.
    ///
    /// Returns the total number of blocks of size `Block_size` that fit in the device.
    ///
    /// # Returns
    ///
    /// The number of blocks in the device.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::Memory_device_type;
    ///
    /// let device = Memory_device_type::<512>::new(2048);
    /// assert_eq!(device.get_block_count(), 4); // 2048 / 512 = 4
    /// ```
    pub fn get_block_count(&self) -> usize {
        let inner = block_on(self.0.read());

        inner.0.len() / BLOCK_SIZE
    }
}

impl<const BLOCK_SIZE: usize> DeviceTrait for MemoryDevice<BLOCK_SIZE> {
    /// Read data from the memory device.
    ///
    /// Reads data from the current position into the provided buffer.
    /// The position is automatically advanced by the number of bytes read.
    fn read(&self, buffer: &mut [u8]) -> crate::Result<Size> {
        let mut inner = self
            .0
            .try_write()
            .map_err(|_| crate::Error::RessourceBusy)?;
        let (data, position) = &mut *inner;

        let read_size = buffer.len().min(data.len().saturating_sub(*position));
        buffer[..read_size].copy_from_slice(&data[*position..*position + read_size]);
        *position += read_size;
        Ok(read_size.into())
    }

    fn write(&self, buffer: &[u8]) -> crate::Result<Size> {
        let mut inner = block_on(self.0.write());
        let (data, position) = &mut *inner;

        let write_size = buffer.len().min(data.len().saturating_sub(*position));
        data[*position..*position + write_size].copy_from_slice(&buffer[..write_size]);
        *position += write_size;

        Ok(write_size.into())
    }

    fn get_size(&self) -> crate::Result<Size> {
        let inner = block_on(self.0.read());

        Ok(Size::new(inner.0.len() as u64))
    }

    fn set_position(&self, position: &Position) -> crate::Result<Size> {
        let mut inner = block_on(self.0.write());
        let (data, device_position) = &mut *inner;

        match position {
            Position::Start(position) => *device_position = *position as usize,
            Position::Current(position) => {
                *device_position = (*device_position as isize + *position as isize) as usize
            }
            Position::End(position) => {
                *device_position = (data.len() as isize - *position as isize) as usize
            }
        }

        Ok(Size::new(*device_position as u64))
    }

    fn erase(&self) -> crate::Result<()> {
        let mut inner = block_on(self.0.write());

        let (data, position) = &mut *inner;

        data[*position..*position + BLOCK_SIZE].fill(0);

        Ok(())
    }

    fn flush(&self) -> crate::Result<()> {
        Ok(())
    }

    fn get_block_size(&self) -> crate::Result<usize> {
        Ok(BLOCK_SIZE)
    }

    fn dump_device(&self) -> crate::Result<Vec<u8>> {
        let inner = block_on(self.0.read());

        Ok(inner.0.clone())
    }

    fn is_a_terminal(&self) -> bool {
        false
    }

    fn is_a_block_device(&self) -> bool {
        false
    }
}
