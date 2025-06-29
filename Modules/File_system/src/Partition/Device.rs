use core::fmt;

use crate::{Device_trait, Device_type, Result_type, Size_type};

/// A device that represents a partition within a larger device
pub struct Partition_device_type {
    Base_device: Device_type,
    Offset: u64,
    Size: u64,
    Position: core::sync::atomic::AtomicU64,
}

impl Partition_device_type {
    /// Create a new partition device
    pub fn New(Base_device: Device_type, Offset: u64, Size: u64) -> Self {
        Self {
            Base_device,
            Offset,
            Size,
            Position: core::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Create a partition device from LBA and sector count
    pub fn New_from_lba(Base_device: Device_type, Start_lba: u32, Sector_count: u32) -> Self {
        let Offset = Start_lba as u64 * 512;
        let Size = Sector_count as u64 * 512;
        Self::New(Base_device, Offset, Size)
    }

    /// Get the byte offset of this partition within the base device
    pub fn Get_offset(&self) -> u64 {
        self.Offset
    }

    /// Get the size of this partition in bytes
    pub fn Get_partition_size(&self) -> u64 {
        self.Size
    }

    /// Get the starting LBA of this partition
    pub fn Get_start_lba(&self) -> u32 {
        (self.Offset / 512) as u32
    }

    /// Get the size in sectors of this partition
    pub fn Get_sector_count(&self) -> u32 {
        (self.Size / 512) as u32
    }

    /// Get the current position within the partition
    pub fn Get_position(&self) -> u64 {
        self.Position.load(core::sync::atomic::Ordering::Relaxed)
    }

    /// Get the base device
    pub fn Get_base_device(&self) -> &Device_type {
        &self.Base_device
    }

    /// Check if the partition device is valid (non-zero size)
    pub fn Is_valid(&self) -> bool {
        self.Size > 0
    }

    /// Get remaining bytes that can be read/written from current position
    pub fn Get_remaining_bytes(&self) -> u64 {
        let Current_pos = self.Get_position();
        self.Size.saturating_sub(Current_pos)
    }

    /// Check if we're at the end of the partition
    pub fn Is_at_end(&self) -> bool {
        self.Get_position() >= self.Size
    }
}

impl Clone for Partition_device_type {
    fn clone(&self) -> Self {
        Self {
            Base_device: self.Base_device.clone(),
            Offset: self.Offset,
            Size: self.Size,
            Position: core::sync::atomic::AtomicU64::new(0), // Reset position for clone
        }
    }
}

impl Device_trait for Partition_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        let Current_pos = self.Position.load(core::sync::atomic::Ordering::Relaxed);

        if Current_pos >= self.Size {
            return Ok(Size_type::New(0));
        }

        let Available = (self.Size - Current_pos).min(Buffer.len() as u64);
        let Read_size = Available as usize;

        // Set position in base device
        let Absolute_pos = self.Offset + Current_pos;
        self.Base_device
            .Set_position(&crate::Position_type::Start(Absolute_pos))?;

        // Read from base device
        let Bytes_read = self.Base_device.Read(&mut Buffer[..Read_size])?;

        // Update position
        self.Position.store(
            Current_pos + Bytes_read.As_u64(),
            core::sync::atomic::Ordering::Relaxed,
        );

        Ok(Bytes_read)
    }

    fn Write(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        let Current_pos = self.Position.load(core::sync::atomic::Ordering::Relaxed);

        if Current_pos >= self.Size {
            return Ok(Size_type::New(0));
        }

        let Available = (self.Size - Current_pos).min(Buffer.len() as u64);
        let Write_size = Available as usize;

        // Set position in base device
        let Absolute_pos = self.Offset + Current_pos;
        self.Base_device
            .Set_position(&crate::Position_type::Start(Absolute_pos))?;

        // Write to base device
        let Bytes_written = self.Base_device.Write(&Buffer[..Write_size])?;

        // Update position
        self.Position.store(
            Current_pos + Bytes_written.As_u64(),
            core::sync::atomic::Ordering::Relaxed,
        );

        Ok(Bytes_written)
    }

    fn Get_size(&self) -> Result_type<Size_type> {
        Ok(Size_type::New(self.Size))
    }

    fn Set_position(&self, Position: &crate::Position_type) -> Result_type<Size_type> {
        use crate::Position_type;

        let New_pos = match Position {
            Position_type::Start(Offset) => *Offset,
            Position_type::Current(Offset) => {
                let Current = self.Position.load(core::sync::atomic::Ordering::Relaxed);
                if *Offset >= 0 {
                    Current.saturating_add(*Offset as u64)
                } else {
                    Current.saturating_sub((-*Offset) as u64)
                }
            }
            Position_type::End(Offset) => {
                if *Offset >= 0 {
                    self.Size.saturating_add(*Offset as u64)
                } else {
                    self.Size.saturating_sub((-*Offset) as u64)
                }
            }
        };

        let Clamped_pos = New_pos.min(self.Size);
        self.Position
            .store(Clamped_pos, core::sync::atomic::Ordering::Relaxed);

        Ok(Size_type::New(Clamped_pos))
    }

    fn Flush(&self) -> Result_type<()> {
        self.Base_device.Flush()
    }

    fn Is_a_block_device(&self) -> bool {
        self.Base_device.Is_a_block_device()
    }

    fn Get_block_size(&self) -> Result_type<usize> {
        self.Base_device.Get_block_size()
    }

    fn Is_a_terminal(&self) -> bool {
        false // Partition devices are never terminals
    }

    fn Erase(&self) -> Result_type<()> {
        // For partition devices, we delegate erase to the base device
        // But we need to be careful about the range
        self.Base_device.Erase()
    }
}

impl fmt::Debug for Partition_device_type {
    fn fmt(&self, Formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Formatter
            .debug_struct("Partition_device_type")
            .field("offset", &self.Offset)
            .field("size", &self.Size)
            .field("start_lba", &self.Get_start_lba())
            .field("sector_count", &self.Get_sector_count())
            .field("position", &self.Get_position())
            .field("remaining_bytes", &self.Get_remaining_bytes())
            .field("is_valid", &self.Is_valid())
            .finish()
    }
}

impl fmt::Display for Partition_device_type {
    fn fmt(&self, Formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            Formatter,
            "Partition Device: Start LBA={}, Sectors={}, Size={} bytes, Position={}/{}",
            self.Get_start_lba(),
            self.Get_sector_count(),
            self.Size,
            self.Get_position(),
            self.Size
        )
    }
}

#[cfg(test)]
mod Tests {
    use super::Partition_device_type;
    use crate::{Device_trait, Device_type, Memory_device_type, Position_type};

    /// Create a mock memory device for testing
    fn Create_test_device() -> Device_type {
        let Memory_device = Memory_device_type::<512>::New(4096);
        crate::Create_device!(Memory_device)
    }

    #[test]
    fn Test_partition_device_creation() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 512, 1024);

        assert_eq!(Partition.Get_offset(), 512);
        assert_eq!(Partition.Get_partition_size(), 1024);
        assert_eq!(Partition.Get_position(), 0);
        assert!(Partition.Is_valid());
    }

    #[test]
    fn Test_partition_device_from_lba() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New_from_lba(Base_device, 4, 8);

        assert_eq!(Partition.Get_offset(), 4 * 512); // 2048
        assert_eq!(Partition.Get_partition_size(), 8 * 512); // 4096
        assert_eq!(Partition.Get_start_lba(), 4);
        assert_eq!(Partition.Get_sector_count(), 8);
    }

    #[test]
    fn Test_partition_device_lba_calculations() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 1024, 2048);

        assert_eq!(Partition.Get_start_lba(), 2); // 1024 / 512
        assert_eq!(Partition.Get_sector_count(), 4); // 2048 / 512
    }

    #[test]
    fn Test_partition_device_validity() {
        let Base_device = Create_test_device();

        let Valid_partition = Partition_device_type::New(Base_device.clone(), 0, 1024);
        assert!(Valid_partition.Is_valid());

        let Invalid_partition = Partition_device_type::New(Base_device, 0, 0);
        assert!(!Invalid_partition.Is_valid());
    }

    #[test]
    fn Test_partition_device_remaining_bytes() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 0, 1000);

        // Initially, all bytes are available
        assert_eq!(Partition.Get_remaining_bytes(), 1000);
        assert!(!Partition.Is_at_end());

        // Simulate advancing position
        let _ = Partition.Set_position(&Position_type::Start(500));
        assert_eq!(Partition.Get_remaining_bytes(), 500);
        assert!(!Partition.Is_at_end());

        // Move to end
        let _ = Partition.Set_position(&Position_type::Start(1000));
        assert_eq!(Partition.Get_remaining_bytes(), 0);
        assert!(Partition.Is_at_end());

        // Beyond end
        let _ = Partition.Set_position(&Position_type::Start(1500));
        assert_eq!(Partition.Get_remaining_bytes(), 0);
        assert!(Partition.Is_at_end());
    }

    #[test]
    fn Test_partition_device_position_setting() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 0, 1000);

        // Test Start position
        let Result = Partition.Set_position(&Position_type::Start(100));
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 100);
        assert_eq!(Partition.Get_position(), 100);

        // Test Current position (positive offset)
        let Result = Partition.Set_position(&Position_type::Current(50));
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 150);
        assert_eq!(Partition.Get_position(), 150);

        // Test Current position (negative offset)
        let Result = Partition.Set_position(&Position_type::Current(-30));
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 120);
        assert_eq!(Partition.Get_position(), 120);

        // Test End position (negative offset)
        let Result = Partition.Set_position(&Position_type::End(-200));
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 800);
        assert_eq!(Partition.Get_position(), 800);

        // Test End position (positive offset) - should clamp to size
        let Result = Partition.Set_position(&Position_type::End(500));
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 1000);
        assert_eq!(Partition.Get_position(), 1000);

        // Test position beyond partition size - should clamp
        let Result = Partition.Set_position(&Position_type::Start(2000));
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 1000);
        assert_eq!(Partition.Get_position(), 1000);
    }

    #[test]
    fn Test_partition_device_get_size() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 100, 1500);

        let Size_result = Partition.Get_size();
        assert!(Size_result.is_ok());
        assert_eq!(Size_result.unwrap().As_u64(), 1500);
    }

    #[test]
    fn Test_partition_device_read() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 0, 100);

        // Test normal read
        let mut Buffer = [0u8; 50];
        let Result = Partition.Read(&mut Buffer);
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 50);
        assert_eq!(Partition.Get_position(), 50);

        // Test read at end of partition
        let mut Buffer = [0u8; 100];
        let Result = Partition.Read(&mut Buffer);
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 50); // Only 50 bytes remaining
        assert_eq!(Partition.Get_position(), 100);

        // Test read beyond end
        let mut Buffer = [0u8; 10];
        let Result = Partition.Read(&mut Buffer);
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 0); // No bytes to read
        assert_eq!(Partition.Get_position(), 100);
    }

    #[test]
    fn Test_partition_device_write() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 0, 100);

        // Test normal write
        let Buffer = [0x42u8; 30];
        let Result = Partition.Write(&Buffer);
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 30);
        assert_eq!(Partition.Get_position(), 30);

        // Test write at end of partition
        let _ = Partition.Set_position(&Position_type::Start(80));
        let Buffer = [0x42u8; 30];
        let Result = Partition.Write(&Buffer);
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 20); // Only 20 bytes available
        assert_eq!(Partition.Get_position(), 100);

        // Test write beyond end
        let Buffer = [0x42u8; 10];
        let Result = Partition.Write(&Buffer);
        assert!(Result.is_ok());
        assert_eq!(Result.unwrap().As_u64(), 0); // No bytes to write
        assert_eq!(Partition.Get_position(), 100);
    }

    #[test]
    fn Test_partition_device_block_operations() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 0, 1000);

        // Test block device properties (should delegate to base device)
        let Is_block = Partition.Is_a_block_device();
        let Block_size = Partition.Get_block_size();

        // These depend on the base device implementation
        // For memory devices, typically not block devices
        assert!(!Is_block);
        assert!(Block_size.is_ok());

        // Test terminal property
        assert!(!Partition.Is_a_terminal());
    }

    #[test]
    fn Test_partition_device_flush_and_erase() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 0, 1000);

        // Test flush (should delegate to base device)
        let Flush_result = Partition.Flush();
        assert!(Flush_result.is_ok());

        // Test erase (should delegate to base device)
        let Erase_result = Partition.Erase();
        assert!(Erase_result.is_ok());
    }

    #[test]
    fn Test_partition_device_debug_display() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New_from_lba(Base_device, 10, 20);

        // Test Debug formatting
        let Debug_str = alloc::format!("{Partition:?}");
        assert!(Debug_str.contains("Partition_device_type"));
        assert!(Debug_str.contains("offset"));
        assert!(Debug_str.contains("size"));
        assert!(Debug_str.contains("start_lba"));

        // Test Display formatting
        let Display_str = alloc::format!("{Partition}");
        assert!(Display_str.contains("Partition Device"));
        assert!(Display_str.contains("Start LBA=10"));
        assert!(Display_str.contains("Sectors=20"));
    }

    #[test]
    fn Test_partition_device_edge_cases() {
        let Base_device = Create_test_device();

        // Test zero offset partition
        let Partition1 = Partition_device_type::New(Base_device.clone(), 0, 512);
        assert_eq!(Partition1.Get_start_lba(), 0);
        assert_eq!(Partition1.Get_sector_count(), 1);

        // Test single byte partition
        let Partition2 = Partition_device_type::New(Base_device.clone(), 512, 1);
        assert_eq!(Partition2.Get_partition_size(), 1);
        assert!(Partition2.Is_valid());

        // Test large LBA values
        let Partition3 = Partition_device_type::New_from_lba(Base_device, 0xFFFFFFFF - 1, 1);
        assert_eq!(Partition3.Get_start_lba(), 0xFFFFFFFF - 1);
        assert_eq!(Partition3.Get_sector_count(), 1);
    }

    #[test]
    fn Test_partition_device_concurrent_access() {
        let Base_device = Create_test_device();
        let Partition = Partition_device_type::New(Base_device, 0, 1000);

        // Test that position is thread-safe (atomic operations)
        let _ = Partition.Set_position(&Position_type::Start(100));
        assert_eq!(Partition.Get_position(), 100);

        // Position should be consistent across multiple reads
        for _ in 0..10 {
            assert_eq!(Partition.Get_position(), 100);
        }
    }

    #[test]
    fn Test_partition_device_clone() {
        let Base_device = Create_test_device();
        let Original = Partition_device_type::New(Base_device, 512, 1024);
        let Cloned = Original.clone();

        // Test that cloned partition has same properties
        assert_eq!(Original.Get_offset(), Cloned.Get_offset());
        assert_eq!(Original.Get_partition_size(), Cloned.Get_partition_size());
        assert_eq!(Original.Get_start_lba(), Cloned.Get_start_lba());
        assert_eq!(Original.Get_sector_count(), Cloned.Get_sector_count());

        // Position should be independent after clone
        let _ = Original.Set_position(&Position_type::Start(100));
        assert_eq!(Original.Get_position(), 100);
        assert_eq!(Cloned.Get_position(), 0); // Cloned device should start at 0
    }
}
