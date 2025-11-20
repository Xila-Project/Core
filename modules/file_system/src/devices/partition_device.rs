//! Partition device implementation for accessing individual partitions.

//!
//! This module provides [`Partition_device_type`], which allows treating individual
//! partitions on a storage device as separate devices. This is essential for file
//! systems that need to operate on specific partitions rather than entire disks.

use core::fmt;

use crate::{
    BaseOperations, DirectBaseOperations, DirectBlockDevice, Error, MountOperations, Position,
    Result, Size, block_device,
};

/// A device implementation that represents a partition within a larger storage device.
///
/// This type wraps a base device and provides access to only a specific region (partition)
/// of that device. It maintains its own position cursor and ensures all operations stay
/// within the partition boundaries. This allows file systems to operate on individual
/// partitions without needing to know about the partition layout.
///
/// # Thread Safety
///
/// The partition device is thread-safe and uses atomic operations for position management.
/// Multiple threads can safely access the same partition device simultaneously.
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use file_system::{MemoryDevice, PartitionDevice, DirectBaseOperations};
///
/// // Create a memory device for testing
/// let base_device = MemoryDevice::<512>::new(1024 * 1024);
///
/// // Create a partition device for blocks 100-199 (100 blocks of 512 bytes = 51.2KB)
/// let partition_device = PartitionDevice::new(&base_device, 100, 100, 512);
///
/// // Now you can use partition_device like any other device
/// let data = b"Hello, Partition!";
/// partition_device.write(data, 0).unwrap();
/// ```
pub struct PartitionDevice<'a, D> {
    /// Base device containing this partition
    base_device: &'a D,
    /// Block size
    block_size: usize,
    /// Byte offset from the beginning of the base device
    start_block: Size,
    /// Size of this partition in bytes
    block_count: Size,
}

impl<'a, D: BaseOperations> PartitionDevice<'a, D> {
    /// Create a new partition device with explicit byte offset and size.
    ///
    /// # Arguments
    ///
    /// * `base_device` - The underlying storage device
    /// * `offset` - Byte offset from the beginning of the base device
    /// * `size` - Size of the partition in bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::{MemoryDevice, PartitionDevice};
    ///
    /// let base_device = MemoryDevice::<512>::new(1024 * 1024);
    /// // Create a partition starting at block 128 (64KB) with 256 blocks (128KB)
    /// let partition = PartitionDevice::new(&base_device, 128, 256, 512);
    /// ```
    pub fn new(
        base_device: &'a D,
        start_block: Size,
        block_count: Size,
        block_size: usize,
    ) -> Self {
        Self {
            base_device,
            block_size,
            start_block,
            block_count,
        }
    }

    /// Get the byte offset of this partition within the base device.
    ///
    /// # Returns
    ///
    /// The absolute byte offset from the beginning of the base device.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::{MemoryDevice, PartitionDevice};
    ///
    /// let base_device = MemoryDevice::<512>::new(1024 * 1024);
    /// let partition = PartitionDevice::new(&base_device, 100, 50, 512);
    /// assert_eq!(partition.get_offset(), 100 * 512);
    /// ```
    pub fn get_offset(&self) -> Size {
        self.start_block * (self.block_size as Size)
    }

    /// Get the size of this partition in bytes.
    ///
    /// # Returns
    ///
    /// The total size of the partition in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::{MemoryDevice, PartitionDevice};
    ///
    /// let base_device = MemoryDevice::<512>::new(1024 * 1024);
    /// let partition = PartitionDevice::new(&base_device, 100, 50, 512);
    /// assert_eq!(partition.get_partition_size(), 50);
    /// ```
    pub fn get_partition_size(&self) -> u64 {
        self.block_count
    }

    /// Get the starting LBA (Logical Block Address) of this partition.
    ///
    /// # Returns
    ///
    /// The sector number where this partition starts (assuming 512-byte sectors).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::{MemoryDevice, PartitionDevice};
    ///
    /// let base_device = MemoryDevice::<512>::new(1024 * 1024);
    /// let partition = PartitionDevice::new(&base_device, 100, 50, 512);
    /// assert_eq!(partition.get_start_lba(), 100);
    /// ```
    pub fn get_start_lba(&self) -> Size {
        self.start_block
    }

    /// Get the size in sectors of this partition.
    ///
    /// # Returns
    ///
    /// The number of 512-byte sectors this partition contains.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::{MemoryDevice, PartitionDevice};
    ///
    /// let base_device = MemoryDevice::<512>::new(1024 * 1024);
    /// let partition = PartitionDevice::new(&base_device, 100, 50, 512);
    /// assert_eq!(partition.get_block_count(), 50);
    /// ```
    pub fn get_block_count(&self) -> u32 {
        self.block_count as u32
    }

    /// Get the base device
    pub fn get_base_device(&self) -> &D {
        self.base_device
    }

    /// Check if the partition device is valid (non-zero size)
    pub fn is_valid(&self) -> bool {
        self.block_count > 0
    }

    pub const fn get_size(&self) -> Size {
        self.block_count * self.block_size as u64
    }
}

impl<'a, D: DirectBaseOperations> DirectBaseOperations for PartitionDevice<'a, D> {
    fn read(&self, buffer: &mut [u8], absolute_position: Size) -> Result<usize> {
        if absolute_position >= self.block_count {
            return Ok(0 as _);
        }

        let read_size = self
            .block_count
            .saturating_sub(absolute_position)
            .min(buffer.len() as u64) as usize;
        let device_position = self.start_block + absolute_position;

        // Read from base device
        let bytes_read = self
            .base_device
            .read(&mut buffer[..read_size], device_position)?;

        Ok(bytes_read)
    }

    fn write(&self, buffer: &[u8], absolute_position: Size) -> Result<usize> {
        if absolute_position >= self.get_size() {
            return Ok(0);
        }

        let write_size = self
            .get_size()
            .saturating_sub(absolute_position)
            .min(buffer.len() as u64) as usize;

        let device_position = self.start_block + absolute_position;

        // Write to base device
        let bytes_written = self
            .base_device
            .write(&buffer[..write_size], device_position)?;

        Ok(bytes_written)
    }

    fn write_pattern(
        &self,
        pattern: &[u8],
        count: usize,
        absolute_position: Size,
    ) -> Result<usize> {
        self.base_device
            .write_pattern(pattern, count, self.start_block + absolute_position)
    }

    fn set_position(&self, current_position: Size, position: &Position) -> Result<Size> {
        let mut position = *position;

        match &mut position {
            Position::Start(offset) => {
                // Clamp to partition size
                if *offset > self.get_size() {
                    return Err(Error::InvalidParameter);
                }

                *offset += self.get_offset();
            }
            Position::Current(offset) => {
                if *offset > 0 {
                    if current_position.saturating_add(*offset as u64) > self.get_size() {
                        return Err(Error::InvalidParameter);
                    }
                } else if current_position.saturating_sub((-*offset) as u64) > self.get_size() {
                    return Err(Error::InvalidParameter);
                }
            }
            Position::End(offset) => {
                if *offset > 0 {
                    return Err(Error::InvalidParameter);
                }

                let end_position = self.get_size() as i64 + *offset;
                if end_position < 0 {
                    return Err(Error::InvalidParameter);
                }
            }
        }

        self.base_device
            .set_position(current_position + self.get_offset(), &position)
    }

    fn flush(&self) -> Result<()> {
        self.base_device.flush()
    }

    fn control(
        &self,
        command: crate::ControlCommand,
        argument: &mut crate::ControlArgument,
    ) -> Result<()> {
        match command {
            block_device::GET_BLOCK_SIZE => {
                *argument
                    .cast::<usize>()
                    .ok_or(crate::Error::InvalidParameter)? = self.block_size;
                Ok(())
            }
            block_device::GET_BLOCK_COUNT => {
                *argument
                    .cast::<Size>()
                    .ok_or(crate::Error::InvalidParameter)? = self.block_count;
                Ok(())
            }
            _ => self.base_device.control(command, argument),
        }
    }
}

impl<'a, D: MountOperations> MountOperations for PartitionDevice<'a, D> {}

impl<'a, D: DirectBlockDevice> DirectBlockDevice for PartitionDevice<'a, D> {}

impl<'a, D> fmt::Debug for PartitionDevice<'a, D>
where
    D: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PartitionDevice")
            .field("offset", &self.start_block)
            .field("size", &self.block_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use crate::{MemoryDevice, implement_block_device_tests};

    use super::*;

    implement_block_device_tests!(PartitionDevice::new(
        Box::leak(Box::new(MemoryDevice::<512>::new(1024 * 1024))),
        0,
        1024 * 512,
        512
    ));
}
