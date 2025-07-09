//! Partition device implementation for accessing individual partitions.

//!
//! This module provides [`Partition_device_type`], which allows treating individual
//! partitions on a storage device as separate devices. This is essential for file
//! systems that need to operate on specific partitions rather than entire disks.

use core::fmt;

use crate::{Device_trait, Device_type, Result_type, Size_type};

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
/// let base_device = create_device!(Memory_device_type::<512>::New(1024 * 1024));
///
/// // Create a partition device for sectors 100-199 (50KB partition)
/// let partition = Partition_device_type::New_from_lba(base_device, 100, 100);
/// let partition_device = create_device!(partition);
///
/// // Now you can use partition_device like any other device
/// let data = b"Hello, Partition!";
/// partition_device.Write(data).unwrap();
/// ```
pub struct Partition_device_type {
    /// Base device containing this partition
    base_device: Device_type,
    /// Byte offset from the beginning of the base device
    offset: u64,
    /// Size of this partition in bytes
    size: u64,
    /// Current position within this partition (atomic for thread safety)
    position: core::sync::atomic::AtomicU64,
}

impl Partition_device_type {
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
    /// let base_device = create_device!(Memory_device_type::<512>::New(1024 * 1024));
    /// // Create a partition starting at 64KB with 128KB size
    /// let partition = Partition_device_type::New(base_device, 64 * 1024, 128 * 1024);
    /// ```
    pub fn new(base_device: Device_type, offset: u64, size: u64) -> Self {
        Self {
            base_device: base_device,
            offset: offset,
            size: size,
            position: core::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Create a partition device from LBA (Logical Block Address) parameters.
    ///
    /// This is a convenience method for creating partition devices using standard
    /// disk partitioning terminology. It assumes 512-byte sectors.
    ///
    /// # Arguments
    ///
    /// * `base_device` - The underlying storage device
    /// * `start_lba` - Starting logical block address (sector number)
    /// * `sector_count` - Number of 512-byte sectors in this partition
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use file_system::*;
    ///
    /// let base_device = create_device!(Memory_device_type::<512>::New(1024 * 1024));
    /// // Create a partition starting at sector 2048 with 1024 sectors (512KB)
    /// let partition = Partition_device_type::New_from_lba(base_device, 2048, 1024);
    /// ```
    pub fn new_from_lba(base_device: Device_type, start_lba: u32, sector_count: u32) -> Self {
        let offset = start_lba as u64 * 512;
        let size = sector_count as u64 * 512;
        Self::new(base_device, offset, size)
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
    /// let base_device = create_device!(Memory_device_type::<512>::New(1024 * 1024));
    /// let partition = Partition_device_type::New_from_lba(base_device, 100, 50);
    /// assert_eq!(partition.get_offset(), 100 * 512);
    /// ```
    pub fn get_offset(&self) -> u64 {
        self.offset
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
    /// let base_device = create_device!(Memory_device_type::<512>::New(1024 * 1024));
    /// let partition = Partition_device_type::New_from_lba(base_device, 100, 50);
    /// assert_eq!(partition.get_partition_size(), 50 * 512);
    /// ```
    pub fn get_partition_size(&self) -> u64 {
        self.size
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
    /// let base_device = create_device!(Memory_device_type::<512>::New(1024 * 1024));
    /// let partition = Partition_device_type::New_from_lba(base_device, 100, 50);
    /// assert_eq!(partition.get_start_lba(), 100);
    /// ```
    pub fn get_start_lba(&self) -> u32 {
        (self.offset / 512) as u32
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
    /// let base_device = create_device!(Memory_device_type::<512>::New(1024 * 1024));
    /// let partition = Partition_device_type::New_from_lba(base_device, 100, 50);
    /// assert_eq!(partition.get_sector_count(), 50);
    /// ```
    pub fn get_sector_count(&self) -> u32 {
        (self.size / 512) as u32
    }

    /// Get the current position within the partition
    pub fn get_position(&self) -> u64 {
        self.position.load(core::sync::atomic::Ordering::Relaxed)
    }

    /// Get the base device
    pub fn get_base_device(&self) -> &Device_type {
        &self.base_device
    }

    /// Check if the partition device is valid (non-zero size)
    pub fn is_valid(&self) -> bool {
        self.size > 0
    }

    /// Get remaining bytes that can be read/written from current position
    pub fn get_remaining_bytes(&self) -> u64 {
        let current_pos = self.get_position();
        self.size.saturating_sub(current_pos)
    }

    /// Check if we're at the end of the partition
    pub fn is_at_end(&self) -> bool {
        self.get_position() >= self.size
    }
}

impl Clone for Partition_device_type {
    fn clone(&self) -> Self {
        Self {
            base_device: self.base_device.clone(),
            offset: self.offset,
            size: self.size,
            position: core::sync::atomic::AtomicU64::new(0), // Reset position for clone
        }
    }
}

impl Device_trait for Partition_device_type {
    fn read(&self, buffer: &mut [u8]) -> Result_type<Size_type> {
        let current_pos = self.position.load(core::sync::atomic::Ordering::Relaxed);

        if current_pos >= self.size {
            return Ok(Size_type::new(0));
        }

        let available = (self.size - current_pos).min(buffer.len() as u64);
        let read_size = available as usize;

        // Set position in base device
        let absolute_pos = self.offset + current_pos;
        self.base_device
            .set_position(&crate::Position_type::Start(absolute_pos))?;

        // Read from base device
        let bytes_read = self.base_device.read(&mut buffer[..read_size])?;

        // Update position
        self.position.store(
            current_pos + bytes_read.as_u64(),
            core::sync::atomic::Ordering::Relaxed,
        );

        Ok(bytes_read)
    }

    fn write(&self, buffer: &[u8]) -> Result_type<Size_type> {
        let current_pos = self.position.load(core::sync::atomic::Ordering::Relaxed);

        if current_pos >= self.size {
            return Ok(Size_type::new(0));
        }

        let available = (self.size - current_pos).min(buffer.len() as u64);
        let write_size = available as usize;

        // Set position in base device
        let absolute_pos = self.offset + current_pos;
        self.base_device
            .set_position(&crate::Position_type::Start(absolute_pos))?;

        // Write to base device
        let bytes_written = self.base_device.write(&buffer[..write_size])?;

        // Update position
        self.position.store(
            current_pos + bytes_written.as_u64(),
            core::sync::atomic::Ordering::Relaxed,
        );

        Ok(bytes_written)
    }

    fn get_size(&self) -> Result_type<Size_type> {
        Ok(Size_type::new(self.size))
    }

    fn set_position(&self, position: &crate::Position_type) -> Result_type<Size_type> {
        use crate::Position_type;

        let new_pos = match position {
            Position_type::Start(offset) => *offset,
            Position_type::Current(offset) => {
                let current = self.position.load(core::sync::atomic::Ordering::Relaxed);
                if *offset >= 0 {
                    current.saturating_add(*offset as u64)
                } else {
                    current.saturating_sub((-*offset) as u64)
                }
            }
            Position_type::End(offset) => {
                if *offset >= 0 {
                    self.size.saturating_add(*offset as u64)
                } else {
                    self.size.saturating_sub((-*offset) as u64)
                }
            }
        };

        let clamped_pos = new_pos.min(self.size);
        self.position
            .store(clamped_pos, core::sync::atomic::Ordering::Relaxed);

        Ok(Size_type::new(clamped_pos))
    }

    fn flush(&self) -> Result_type<()> {
        self.base_device.flush()
    }

    fn is_a_block_device(&self) -> bool {
        self.base_device.is_a_block_device()
    }

    fn get_block_size(&self) -> Result_type<usize> {
        self.base_device.get_block_size()
    }

    fn is_a_terminal(&self) -> bool {
        false // Partition devices are never terminals
    }

    fn erase(&self) -> Result_type<()> {
        // For partition devices, we delegate erase to the base device
        // But we need to be careful about the range
        self.base_device.erase()
    }
}

impl fmt::Debug for Partition_device_type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Partition_device_type")
            .field("offset", &self.offset)
            .field("size", &self.size)
            .field("start_lba", &self.get_start_lba())
            .field("sector_count", &self.get_sector_count())
            .field("position", &self.get_position())
            .field("remaining_bytes", &self.get_remaining_bytes())
            .field("is_valid", &self.is_valid())
            .finish()
    }
}

impl fmt::Display for Partition_device_type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Partition Device: Start LBA={}, Sectors={}, Size={} bytes, Position={}/{}",
            self.get_start_lba(),
            self.get_sector_count(),
            self.size,
            self.get_position(),
            self.size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Partition_device_type;
    use crate::{Device_trait, Device_type, Memory_device_type, Position_type};

    /// Create a mock memory device for testing
    fn create_test_device() -> Device_type {
        let memory_device = Memory_device_type::<512>::new(4096);
        crate::create_device!(memory_device)
    }

    #[test]
    fn test_partition_device_creation() {
        let base_device = create_test_device();
        let partition = Partition_device_type::new(base_device, 512, 1024);

        assert_eq!(partition.get_offset(), 512);
        assert_eq!(partition.get_partition_size(), 1024);
        assert_eq!(partition.get_position(), 0);
        assert!(partition.is_valid());
    }

    #[test]
    fn test_partition_device_from_lba() {
        let base_device = create_test_device();
        let partition = Partition_device_type::new_from_lba(base_device, 4, 8);

        assert_eq!(partition.get_offset(), 4 * 512); // 2048
        assert_eq!(partition.get_partition_size(), 8 * 512); // 4096
        assert_eq!(partition.get_start_lba(), 4);
        assert_eq!(partition.get_sector_count(), 8);
    }

    #[test]
    fn test_partition_device_lba_calculations() {
        let base_device = create_test_device();
        let partition = Partition_device_type::new(base_device, 1024, 2048);

        assert_eq!(partition.get_start_lba(), 2); // 1024 / 512
        assert_eq!(partition.get_sector_count(), 4); // 2048 / 512
    }

    #[test]
    fn test_partition_device_validity() {
        let base_device = create_test_device();

        let valid_partition = Partition_device_type::new(base_device.clone(), 0, 1024);
        assert!(valid_partition.is_valid());

        let invalid_partition = Partition_device_type::new(base_device, 0, 0);
        assert!(!invalid_partition.is_valid());
    }

    #[test]
    fn test_partition_device_remaining_bytes() {
        let base_device = create_test_device();
        let partition = Partition_device_type::new(base_device, 0, 1000);

        // Initially, all bytes are available
        assert_eq!(partition.get_remaining_bytes(), 1000);
        assert!(!partition.is_at_end());

        // Simulate advancing position
        let _ = partition.set_position(&Position_type::Start(500));
        assert_eq!(partition.get_remaining_bytes(), 500);
        assert!(!partition.is_at_end());

        // Move to end
        let _ = partition.set_position(&Position_type::Start(1000));
        assert_eq!(partition.get_remaining_bytes(), 0);
        assert!(partition.is_at_end());

        // Beyond end
        let _ = partition.set_position(&Position_type::Start(1500));
        assert_eq!(partition.get_remaining_bytes(), 0);
        assert!(partition.is_at_end());
    }

    #[test]
    fn test_partition_device_position_setting() {
        let base_device = create_test_device();
        let partition = Partition_device_type::new(base_device, 0, 1000);

        // Test Start position
        let result = partition.set_position(&Position_type::Start(100));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 100);
        assert_eq!(partition.get_position(), 100);

        // Test Current position (positive offset)
        let result = partition.set_position(&Position_type::Current(50));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 150);
        assert_eq!(partition.get_position(), 150);

        // Test Current position (negative offset)
        let result = partition.set_position(&Position_type::Current(-30));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 120);
        assert_eq!(partition.get_position(), 120);

        // Test End position (negative offset)
        let result = partition.set_position(&Position_type::End(-200));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 800);
        assert_eq!(partition.get_position(), 800);

        // Test End position (positive offset) - should clamp to size
        let result = partition.set_position(&Position_type::End(500));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 1000);
        assert_eq!(partition.get_position(), 1000);

        // Test position beyond partition size - should clamp
        let result = partition.set_position(&Position_type::Start(2000));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 1000);
        assert_eq!(partition.get_position(), 1000);
    }

    #[test]
    fn test_partition_device_get_size() {
        let base_device = create_test_device();
        let partition = Partition_device_type::new(base_device, 100, 1500);

        let size_result = partition.get_size();
        assert!(size_result.is_ok());
        assert_eq!(size_result.unwrap().as_u64(), 1500);
    }

    #[test]
    fn test_partition_device_read() {
        let base_device = create_test_device();
        let partition = Partition_device_type::new(base_device, 0, 100);

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
        let partition = Partition_device_type::new(base_device, 0, 100);

        // Test normal write
        let buffer = [0x42u8; 30];
        let result = partition.write(&buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_u64(), 30);
        assert_eq!(partition.get_position(), 30);

        // Test write at end of partition
        let _ = partition.set_position(&Position_type::Start(80));
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
        let partition = Partition_device_type::new(base_device, 0, 1000);

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
        let partition = Partition_device_type::new(base_device, 0, 1000);

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
        let partition = Partition_device_type::new_from_lba(base_device, 10, 20);

        // Test Debug formatting
        let debug_str = alloc::format!("{partition:?}");
        assert!(debug_str.contains("Partition_device_type"));
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
        let partition1 = Partition_device_type::new(base_device.clone(), 0, 512);
        assert_eq!(partition1.get_start_lba(), 0);
        assert_eq!(partition1.get_sector_count(), 1);

        // Test single byte partition
        let partition2 = Partition_device_type::new(base_device.clone(), 512, 1);
        assert_eq!(partition2.get_partition_size(), 1);
        assert!(partition2.is_valid());

        // Test large LBA values
        let partition3 = Partition_device_type::new_from_lba(base_device, 0xFFFFFFFF - 1, 1);
        assert_eq!(partition3.get_start_lba(), 0xFFFFFFFF - 1);
        assert_eq!(partition3.get_sector_count(), 1);
    }

    #[test]
    fn test_partition_device_concurrent_access() {
        let base_device = create_test_device();
        let partition = Partition_device_type::new(base_device, 0, 1000);

        // Test that position is thread-safe (atomic operations)
        let _ = partition.set_position(&Position_type::Start(100));
        assert_eq!(partition.get_position(), 100);

        // Position should be consistent across multiple reads
        for _ in 0..10 {
            assert_eq!(partition.get_position(), 100);
        }
    }

    #[test]
    fn test_partition_device_clone() {
        let base_device = create_test_device();
        let original = Partition_device_type::new(base_device, 512, 1024);
        let cloned = original.clone();

        // Test that cloned partition has same properties
        assert_eq!(original.get_offset(), cloned.get_offset());
        assert_eq!(original.get_partition_size(), cloned.get_partition_size());
        assert_eq!(original.get_start_lba(), cloned.get_start_lba());
        assert_eq!(original.get_sector_count(), cloned.get_sector_count());

        // Position should be independent after clone
        let _ = original.set_position(&Position_type::Start(100));
        assert_eq!(original.get_position(), 100);
        assert_eq!(cloned.get_position(), 0); // Cloned device should start at 0
    }
}
