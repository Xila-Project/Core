//! Partition device implementation for accessing individual partitions.

//!
//! This module provides [`PartitionDevice`], which allows treating individual
//! partitions on a storage device as separate devices. This is essential for file
//! systems that need to operate on specific partitions rather than entire disks.

use core::fmt;

use shared::AnyByLayout;

use crate::{
    BaseOperations, ControlCommand, ControlCommandIdentifier, DirectBaseOperations,
    DirectBlockDevice, Error, MountOperations, Position, Result, Size,
    block_device::{self, GET_BLOCK_COUNT, GET_BLOCK_SIZE},
};

/// A device implementation that represents a partition within a larger storage device.
///
/// This type wraps a base device and provides access to only a specific region (partition)
/// of that device. It maintains its own position cursor and ensures all operations stay
/// within the partition boundaries. This allows file systems to operate on individual
/// partitions without needing to know about the partition layout.
///
///
/// Note: The partition device does not manage position state internally and does not use atomic operations.
/// Thread safety depends on the underlying base device implementation.
///
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
    block_size: u32,
    /// Byte offset from the beginning of the base device
    offset: Size,
    /// Size of this partition in bytes
    size: Size,
}

impl<'a, D: BaseOperations> PartitionDevice<'a, D> {
    /// Create a new partition device with explicit byte offset and size.
    ///
    /// # Arguments
    ///
    /// * `base_device` - The underlying storage device
    /// * `start_block` - Block index where the partition starts
    /// * `block_count` - Number of blocks in the partition
    /// * `block_size` - Size of each block in bytes
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
    pub fn new(base_device: &'a D, start_block: Size, block_count: Size, block_size: u32) -> Self {
        Self {
            base_device,
            block_size,
            offset: start_block * (block_size as Size),
            size: block_count * (block_size as Size),
        }
    }
    /// Get the number of blocks in this partition.
    ///
    /// # Returns
    ///
    /// The number of blocks in this partition, where each block is of the partition's configured block size.
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
    pub const fn get_block_count(&self) -> u32 {
        self.size as u32 / self.block_size
    }

    pub const fn get_start_lba(&self) -> Size {
        self.offset / (self.block_size as Size)
    }

    /// Get the base device
    pub const fn get_base_device(&self) -> &D {
        self.base_device
    }

    /// Check if the partition device is valid (non-zero size)
    pub const fn is_valid(&self) -> bool {
        self.size > 0
    }

    const fn get_device_position(&self, absolute_position: Size) -> Option<Size> {
        if absolute_position >= self.size {
            return None;
        }

        let device_position = match self.offset.checked_add(absolute_position) {
            Some(pos) => pos,
            None => return None,
        };

        Some(device_position)
    }

    const fn get_total_buffer_length(
        &self,
        absolute_position: Size,
        buffer_length: usize,
    ) -> usize {
        let remaining_size = self.size.saturating_sub(absolute_position) as usize;

        if buffer_length > remaining_size {
            remaining_size
        } else {
            buffer_length
        }
    }
}

impl<'a, D: DirectBaseOperations> DirectBaseOperations for PartitionDevice<'a, D> {
    fn read(&self, buffer: &mut [u8], absolute_position: Size) -> Result<usize> {
        let device_position = match self.get_device_position(absolute_position) {
            Some(pos) => pos,
            None => return Ok(0),
        };

        let read_size = self.get_total_buffer_length(absolute_position, buffer.len());

        // Read from base device
        let bytes_read = self
            .base_device
            .read(&mut buffer[..read_size], device_position)?;

        Ok(bytes_read)
    }

    fn write(&self, buffer: &[u8], absolute_position: Size) -> Result<usize> {
        let device_position = match self.get_device_position(absolute_position) {
            Some(pos) => pos,
            None => return Ok(0),
        };

        let write_size = self.get_total_buffer_length(absolute_position, buffer.len());

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
        let device_position = match self.get_device_position(absolute_position) {
            Some(pos) => pos,
            None => return Ok(0),
        };

        let total_write_size = pattern.len() * count;
        let maximum_write_size = self.get_total_buffer_length(absolute_position, total_write_size);

        let adjusted_count = if total_write_size > maximum_write_size {
            // Adjust count to fit within partition boundaries
            maximum_write_size / pattern.len()
        } else {
            count
        };

        self.base_device
            .write_pattern(pattern, adjusted_count, device_position)
    }

    fn set_position(&self, current_position: Size, position: &Position) -> Result<Size> {
        let position = block_device::set_position(current_position, position, self.size)?;

        let device_position = self
            .get_device_position(position)
            .ok_or(Error::InvalidParameter)?;

        self.base_device
            .set_position(device_position, &Position::Start(0))?;

        Ok(position)
    }

    fn flush(&self) -> Result<()> {
        self.base_device.flush()
    }

    fn control(
        &self,
        command: ControlCommandIdentifier,
        input: &AnyByLayout,
        output: &mut AnyByLayout,
    ) -> Result<()> {
        match command {
            GET_BLOCK_SIZE::IDENTIFIER => {
                let output = GET_BLOCK_SIZE::cast_output(output)?;

                *output = self.block_size;
            }
            GET_BLOCK_COUNT::IDENTIFIER => {
                *output
                    .cast_mutable::<Size>()
                    .ok_or(Error::InvalidParameter)? = self.get_block_count() as Size;
            }
            _ => return self.base_device.control(command, input, output),
        }

        Ok(())
    }
}

impl<'a, D: MountOperations> MountOperations for PartitionDevice<'a, D> {}

impl<'a, D: DirectBlockDevice> DirectBlockDevice for PartitionDevice<'a, D> {}

impl<'a, D> fmt::Debug for PartitionDevice<'a, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PartitionDevice")
            .field("block_size", &self.block_size)
            .field("offset", &self.offset)
            .field("size", &self.size)
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
