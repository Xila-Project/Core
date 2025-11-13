//! Partition device implementation for accessing individual partitions.

//!
//! This module provides [`Partition_device_type`], which allows treating individual
//! partitions on a storage device as separate devices. This is essential for file
//! systems that need to operate on specific partitions rather than entire disks.

use core::fmt;

use crate::{BaseOperations, DirectBlockDevice, DirectFileOperations, Position, Result, Size};

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
/// # use file_system::*;
///
/// // Create a memory device for testing
/// let base_device = create_device!(MemoryDevice::<512>::new(1024 * 1024));
///
/// // Create a partition device for sectors 100-199 (50KB partition)
/// let partition = PartitionDevice::new_from_lba(base_device, 100, 100).unwrap();
/// let partition_device = create_device!(partition);
///
/// // Now you can use partition_device like any other device
/// let data = b"Hello, Partition!";
/// partition_device.write(data).unwrap();
/// ```
pub struct PartitionDevice<'a, D> {
    /// Base device containing this partition
    base_device: &'a D,
    /// Block size
    block_size: usize,
    /// Byte offset from the beginning of the base device
    block_offset: Size,
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
    /// # use file_system::*;
    ///
    /// let base_device = create_device!(MemoryDevice::<512>::new(1024 * 1024));
    /// // Create a partition starting at 64KB with 128KB size
    /// let partition = PartitionDevice::new(base_device, 64 * 1024, 128 * 1024, 512);
    /// ```
    pub fn new(
        base_device: &'a D,
        block_offset: Size,
        block_count: Size,
        block_size: usize,
    ) -> Self {
        Self {
            base_device,
            block_size,
            block_offset,
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
    /// # use file_system::*;
    ///
    /// let base_device = create_device!(MemoryDevice::<512>::new(1024 * 1024));
    /// let partition = PartitionDevice::new_from_lba(base_device, 100, 50).unwrap();
    /// assert_eq!(partition.get_offset(), 100 * 512);
    /// ```
    pub fn get_offset(&self) -> u64 {
        self.block_offset
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
    /// # use file_system::*;
    ///
    /// let base_device = create_device!(MemoryDevice::<512>::new(1024 * 1024));
    /// let partition = PartitionDevice::new_from_lba(base_device, 100, 50).unwrap();
    /// assert_eq!(partition.get_partition_size(), 50 * 512);
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
    /// # use file_system::*;
    ///
    /// let base_device = create_device!(MemoryDevice::<512>::new(1024 * 1024));
    /// let partition = PartitionDevice::new_from_lba(base_device, 100, 50).unwrap();
    /// assert_eq!(partition.get_start_lba(), 100);
    /// ```
    pub fn get_start_lba(&self) -> u32 {
        (self.block_offset / self.block_size as u64) as u32
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
    /// # use file_system::*;
    ///
    /// let base_device = create_device!(MemoryDevice::<512>::new(1024 * 1024));
    /// let partition = PartitionDevice::new_from_lba(base_device, 100, 50).unwrap();
    /// assert_eq!(partition.get_sector_count(), 50);
    /// ```
    pub fn get_sector_count(&self) -> u32 {
        (self.block_count / self.block_size as u64) as u32
    }

    /// Get the base device
    pub fn get_base_device(&self) -> &D {
        &self.base_device
    }

    /// Check if the partition device is valid (non-zero size)
    pub fn is_valid(&self) -> bool {
        self.block_count > 0
    }

    pub const fn get_size(&self) -> Size {
        self.block_count * self.block_size as u64
    }
}

impl<'a, D: DirectFileOperations> DirectFileOperations for PartitionDevice<'a, D> {
    fn read(&self, buffer: &mut [u8], absolute_position: Size) -> Result<usize> {
        if absolute_position >= self.block_count {
            return Ok(0 as _);
        }

        let read_size = self
            .block_count
            .saturating_sub(absolute_position)
            .min(buffer.len() as u64) as usize;
        let device_position = self.block_offset + absolute_position;

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

        let device_position = self.block_offset + absolute_position;

        // Write to base device
        let bytes_written = self
            .base_device
            .write(&buffer[..write_size], device_position)?;

        Ok(bytes_written)
    }

    fn set_position(&self, position: &crate::Position) -> Result<Size> {
        let mut position = position.clone();
        if let Position::Start(offset) = &mut position {
            *offset += self.block_offset;
        }

        let new_device_position = self.base_device.set_position(&position)?;

        let new_partition_position = new_device_position
            .saturating_sub(self.block_offset)
            .min(self.get_size());

        Ok(new_partition_position)
    }

    fn flush(&self) -> Result<()> {
        self.base_device.flush()
    }
}

impl<'a, D: DirectBlockDevice> DirectBlockDevice for PartitionDevice<'a, D> {
    fn get_size(&self) -> Result<Size> {
        Ok(self.block_count * self.block_size as u64)
    }

    fn get_block_size(&self) -> Result<usize> {
        Ok(self.block_size)
    }

    fn erase(&self, mut absolute_position: Size) -> Result<()> {
        absolute_position += self.block_offset;

        self.base_device.erase(absolute_position)
    }

    fn get_block_count(&self) -> Result<Size> {
        Ok(self.block_count)
    }
}

impl<'a, D> fmt::Debug for PartitionDevice<'a, D>
where
    D: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PartitionDevice")
            .field("offset", &self.block_offset)
            .field("size", &self.block_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::PartitionDevice;
    use crate::{BaseOperations, Device, MemoryDevice, Position};

    const BLOCK_SIZE: usize = 512;

    /// Create a mock memory device for testing
    fn create_test_device() -> Device {
        let memory_device = MemoryDevice::<512>::new(4096);
        crate::create_device!(memory_device)
    }

    #[test]
    fn test_partition_device_creation() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 512, 1024, BLOCK_SIZE);

        assert_eq!(partition.get_offset(), 512);
        assert_eq!(partition.get_partition_size(), 1024);
        assert_eq!(partition.get_position(), 0);
        assert!(partition.is_valid());
    }

    #[test]
    fn test_partition_device_from_lba() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new_from_lba(base_device, 4, 8).unwrap();

        assert_eq!(partition.get_offset(), 4 * 512); // 2048
        assert_eq!(partition.get_partition_size(), 8 * 512); // 4096
        assert_eq!(partition.get_start_lba(), 4);
        assert_eq!(partition.get_sector_count(), 8);
    }

    #[test]
    fn test_partition_device_lba_calculations() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 1024, 2048, BLOCK_SIZE);

        assert_eq!(partition.get_start_lba(), 2); // 1024 / 512
        assert_eq!(partition.get_sector_count(), 4); // 2048 / 512
    }

    #[test]
    fn test_partition_device_validity() {
        let base_device = create_test_device();

        let valid_partition = PartitionDevice::new(base_device.clone(), 0, 1024, BLOCK_SIZE);
        assert!(valid_partition.is_valid());

        let invalid_partition = PartitionDevice::new(base_device, 0, 0, BLOCK_SIZE);
        assert!(!invalid_partition.is_valid());
    }

    #[test]
    fn test_partition_device_remaining_bytes() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 0, 1000, BLOCK_SIZE);

        // Initially, all bytes are available
        assert_eq!(partition.get_remaining_bytes(), 1000);
        assert!(!partition.is_at_end());

        // Simulate advancing position
        let _ = partition.set_position(&Position::Start(500));
        assert_eq!(partition.get_remaining_bytes(), 500);
        assert!(!partition.is_at_end());

        // Move to end
        let _ = partition.set_position(&Position::Start(1000));
        assert_eq!(partition.get_remaining_bytes(), 0);
        assert!(partition.is_at_end());

        // Beyond end
        let _ = partition.set_position(&Position::Start(1500));
        assert_eq!(partition.get_remaining_bytes(), 0);
        assert!(partition.is_at_end());
    }

    #[test]
    fn test_partition_device_position_setting() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 0, 1000, BLOCK_SIZE);

        // Test Start position
        let result = partition.set_position(&Position::Start(100));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 100);
        assert_eq!(partition.get_position(), 100);

        // Test Current position (positive offset)
        let result = partition.set_position(&Position::Current(50));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 150);
        assert_eq!(partition.get_position(), 150);

        // Test Current position (negative offset)
        let result = partition.set_position(&Position::Current(-30));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 120);
        assert_eq!(partition.get_position(), 120);

        // Test End position (negative offset)
        let result = partition.set_position(&Position::End(-200));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 800);
        assert_eq!(partition.get_position(), 800);

        // Test End position (positive offset) - should clamp to size
        let result = partition.set_position(&Position::End(500));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 1000);
        assert_eq!(partition.get_position(), 1000);

        // Test position beyond partition size - should clamp
        let result = partition.set_position(&Position::Start(2000));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 1000);
        assert_eq!(partition.get_position(), 1000);
    }

    #[test]
    fn test_partition_device_get_size() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 100, 1500, BLOCK_SIZE);

        let size_result = partition.get_size();
        assert!(size_result.is_ok());
        assert_eq!(size_result.unwrap().as_u64(), 1500);
    }

    #[test]
    fn test_partition_device_read() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 0, 100, BLOCK_SIZE);

        // Test normal read
        let mut buffer = [0u8; 50];
        let result = partition.read(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 50);
        assert_eq!(partition.get_position(), 50);

        // Test read at end of partition
        let mut buffer = [0u8; 100];
        let result = partition.read(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 50); // Only 50 bytes remaining
        assert_eq!(partition.get_position(), 100);

        // Test read beyond end
        let mut buffer = [0u8; 10];
        let result = partition.read(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 0); // No bytes to read
        assert_eq!(partition.get_position(), 100);
    }

    #[test]
    fn test_partition_device_write() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 0, 100, BLOCK_SIZE);

        // Test normal write
        let buffer = [0x42u8; 30];
        let result = partition.write(&buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 30);
        assert_eq!(partition.get_position(), 30);

        // Test write at end of partition
        let _ = partition.set_position(&Position::Start(80));
        let buffer = [0x42u8; 30];
        let result = partition.write(&buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 20); // Only 20 bytes available
        assert_eq!(partition.get_position(), 100);

        // Test write beyond end
        let buffer = [0x42u8; 10];
        let result = partition.write(&buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 0); // No bytes to write
        assert_eq!(partition.get_position(), 100);
    }

    #[test]
    fn test_partition_device_block_operations() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 0, 1000, BLOCK_SIZE);

        // Test block device properties (should delegate to base device)
        let is_block = partition.is_a_block_device();
        let block_size = partition.get_block_size();

        // These depend on the base device implementation
        // For memory devices, typically not block devices
        assert!(!is_block);
        assert!(block_size.is_ok());

        // Test terminal property
        assert!(!partition.is_a_terminal());
    }

    #[test]
    fn test_partition_device_flush_and_erase() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 0, 1000, BLOCK_SIZE);

        // Test flush (should delegate to base device)
        let flush_result = partition.flush();
        assert!(flush_result.is_ok());

        // Test erase (should delegate to base device)
        let erase_result = partition.erase();
        assert!(erase_result.is_ok());
    }

    #[test]
    fn test_partition_device_debug_display() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new_from_lba(base_device, 10, 20).unwrap();

        // Test Debug formatting
        let debug_str = alloc::format!("{partition:?}");
        assert!(debug_str.contains("PartitionDevice"));
        assert!(debug_str.contains("offset"));
        assert!(debug_str.contains("size"));
        assert!(debug_str.contains("start_lba"));

        // Test Display formatting
        let display_str = alloc::format!("{partition}");
        assert!(display_str.contains("Partition Device"));
        assert!(display_str.contains("Start LBA=10"));
        assert!(display_str.contains("Sectors=20"));
    }

    #[test]
    fn test_partition_device_edge_cases() {
        let base_device = create_test_device();

        // Test zero offset partition
        let partition1 = PartitionDevice::new(base_device.clone(), 0, 512, BLOCK_SIZE);
        assert_eq!(partition1.get_start_lba(), 0);
        assert_eq!(partition1.get_sector_count(), 1);

        // Test single byte partition
        let partition2 = PartitionDevice::new(base_device.clone(), 512, 1, BLOCK_SIZE);
        assert_eq!(partition2.get_partition_size(), 1);
        assert!(partition2.is_valid());

        // Test large LBA values
        let partition3 = PartitionDevice::new_from_lba(base_device, 0xFFFFFFFF - 1, 1).unwrap();
        assert_eq!(partition3.get_start_lba(), 0xFFFFFFFF - 1);
        assert_eq!(partition3.get_sector_count(), 1);
    }

    #[test]
    fn test_partition_device_concurrent_access() {
        let base_device = create_test_device();
        let partition = PartitionDevice::new(base_device, 0, 1000, BLOCK_SIZE);

        // Test that position is thread-safe (atomic operations)
        let _ = partition.set_position(&Position::Start(100));
        assert_eq!(partition.get_position(), 100);

        // Position should be consistent across multiple reads
        for _ in 0..10 {
            assert_eq!(partition.get_position(), 100);
        }
    }

    #[test]
    fn test_partition_device_clone() {
        let base_device = create_test_device();
        let original = PartitionDevice::new(base_device, 512, 1024, BLOCK_SIZE);
        let cloned = original.clone();

        // Test that cloned partition has same properties
        assert_eq!(original.get_offset(), cloned.get_offset());
        assert_eq!(original.get_partition_size(), cloned.get_partition_size());
        assert_eq!(original.get_start_lba(), cloned.get_start_lba());
        assert_eq!(original.get_sector_count(), cloned.get_sector_count());

        // Position should be independent after clone
        let _ = original.set_position(&Position::Start(100));
        assert_eq!(original.get_position(), 100);
        assert_eq!(cloned.get_position(), 0); // Cloned device should start at 0
    }
}
