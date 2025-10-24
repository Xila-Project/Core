//! Partition entry structures for MBR partition tables.
//!
//! This module provides the [`PartitionEntry`] structure which represents
//! individual partition entries in Master Boot Record (MBR) partition tables.
//! Each entry contains information about a partition's location, size, type, and bootability.

use core::fmt;

/// MBR partition table entry structure (16 bytes).
///
/// This structure represents a single partition entry in an MBR partition table.
/// Each MBR contains exactly 4 partition entries, defining up to 4 primary partitions.
/// The structure follows the traditional PC BIOS partition table format.
///
/// # Memory Layout
///
/// The structure is packed and has a fixed 16-byte layout for MBR compatibility:
/// - Bytes 0: Boot indicator
/// - Bytes 1-3: CHS start address (legacy)
/// - Byte 4: Partition type ID
/// - Bytes 5-7: CHS end address (legacy)
/// - Bytes 8-11: LBA start address (little-endian)
/// - Bytes 12-15: Partition size in sectors (little-endian)
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// use file_system::*;
///
/// // Create a new bootable FAT32 partition
/// let partition = PartitionEntry::new_with_params(
///     true,
///     PartitionKind::Fat32Lba,
///     2048,
///     204800
/// );
///
/// assert!(partition.is_bootable());
/// assert_eq!(partition.get_start_lba(), 2048);
/// assert_eq!(partition.get_size_sectors(), 204800);
/// ```
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct PartitionEntry {
    /// Boot indicator (0x80 = bootable, 0x00 = non-bootable)
    pub bootable: u8,
    /// Starting head
    pub start_head: u8,
    /// Starting sector (bits 5-0) and cylinder high bits (bits 7-6)
    pub start_sector: u8,
    /// Starting cylinder (low 8 bits)
    pub start_cylinder: u8,
    /// Partition type ID
    pub partition_type: u8,
    /// Ending head
    pub end_head: u8,
    /// Ending sector (bits 5-0) and cylinder high bits (bits 7-6)
    pub end_sector: u8,
    /// Ending cylinder (low 8 bits)
    pub end_cylinder: u8,
    /// Starting LBA (Logical Block Address)
    pub start_lba: u32,
    /// Size in sectors
    pub size_sectors: u32,
}

impl PartitionEntry {
    /// Create a new empty (invalid) partition entry.
    ///
    /// All fields are initialized to zero, making this an invalid partition entry
    /// that will not be recognized by the MBR parser.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// use file_system::*;
    ///
    /// let partition = PartitionEntry::new();
    /// assert!(!partition.is_valid());
    /// assert!(!partition.is_bootable());
    /// ```
    pub fn new() -> Self {
        Self {
            bootable: 0,
            start_head: 0,
            start_sector: 0,
            start_cylinder: 0,
            partition_type: 0,
            end_head: 0,
            end_sector: 0,
            end_cylinder: 0,
            start_lba: 0,
            size_sectors: 0,
        }
    }

    /// Create a new partition entry with specified parameters.
    ///
    /// This constructor creates a valid partition entry with the specified type,
    /// location, and size. The CHS (Cylinder-Head-Sector) fields are not set
    /// as modern systems use LBA addressing.
    ///
    /// # Arguments
    ///
    /// * `Bootable` - Whether this partition should be marked as bootable
    /// * `Partition_type` - The type of partition (FAT32, Linux, etc.)
    /// * `Start_lba` - Starting logical block address (sector number)
    /// * `Size_sectors` - Size of the partition in 512-byte sectors
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// use file_system::*;
    ///
    /// // Create a 100MB FAT32 partition starting at sector 2048
    /// let partition = PartitionEntry::new_with_params(
    ///     true,
    ///     PartitionKind::Fat32Lba,
    ///     2048,
    ///     204800
    /// );
    ///
    /// assert!(partition.is_valid());
    /// assert!(partition.is_bootable());
    /// ```
    pub fn new_with_params(
        bootable: bool,
        partition_type: crate::PartitionKind,
        start_lba: u32,
        size_sectors: u32,
    ) -> Self {
        let mut entry = Self::new();
        entry.bootable = if bootable { 0x80 } else { 0x00 };
        entry.set_partition_type(partition_type);
        entry.start_lba = start_lba.to_le();
        entry.size_sectors = size_sectors.to_le();
        entry
    }

    /// Check if this partition entry is valid (non-zero)
    pub fn is_valid(&self) -> bool {
        self.partition_type != 0 && self.size_sectors > 0
    }

    /// Check if this partition is bootable
    pub fn is_bootable(&self) -> bool {
        self.bootable == 0x80
    }

    /// Set the bootable flag
    pub fn set_bootable(&mut self, bootable: bool) {
        self.bootable = if bootable { 0x80 } else { 0x00 };
    }

    /// Get the starting LBA of this partition
    pub fn get_start_lba(&self) -> u32 {
        u32::from_le(self.start_lba)
    }

    /// Set the starting LBA of this partition
    pub fn set_start_lba(&mut self, start_lba: u32) {
        self.start_lba = start_lba.to_le();
    }

    /// Get the size in sectors of this partition
    pub fn get_size_sectors(&self) -> u32 {
        u32::from_le(self.size_sectors)
    }

    /// Set the size in sectors of this partition
    pub fn set_size_sectors(&mut self, size_sectors: u32) {
        self.size_sectors = size_sectors.to_le();
    }

    /// Get the partition type as an enum
    pub fn get_partition_type(&self) -> crate::PartitionKind {
        crate::PartitionKind::from_u8(self.partition_type)
    }

    /// Set the partition type from an enum
    pub fn set_partition_type(&mut self, partition_type: crate::PartitionKind) {
        self.partition_type = partition_type.to_u8();
    }

    /// Get the partition type as a human-readable string
    pub fn get_partition_type_name(&self) -> &'static str {
        self.get_partition_type().get_name()
    }

    /// Get the end LBA of this partition (start + size - 1)
    pub fn get_end_lba(&self) -> u32 {
        self.get_start_lba() + self.get_size_sectors() - 1
    }

    /// Get the size in bytes of this partition
    pub fn get_size_bytes(&self) -> u64 {
        self.get_size_sectors() as u64 * 512
    }

    /// Check if this partition overlaps with another partition
    pub fn overlaps_with(&self, other: &Self) -> bool {
        if !self.is_valid() || !other.is_valid() {
            return false;
        }

        let self_start = self.get_start_lba();
        let self_end = self.get_end_lba();
        let other_start = other.get_start_lba();
        let other_end = other.get_end_lba();

        !(self_end < other_start || other_end < self_start)
    }

    /// Check if a given LBA is within this partition
    pub fn contains_lba(&self, lba: u32) -> bool {
        if !self.is_valid() {
            return false;
        }

        let start = self.get_start_lba();
        let end = self.get_end_lba();
        lba >= start && lba <= end
    }

    /// Clear the partition entry (make it empty)
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

impl Default for PartitionEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PartitionEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_valid() {
            write!(formatter, "Empty partition")
        } else {
            write!(
                formatter,
                "Partition: Type={:02X} ({}), Start_LBA={}, Size={} sectors ({} MB), Bootable={}",
                self.partition_type,
                self.get_partition_type_name(),
                self.get_start_lba(),
                self.get_size_sectors(),
                self.get_size_bytes() / (1024 * 1024),
                self.is_bootable()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PartitionEntry;
    use crate::PartitionKind;
    use alloc::format;

    fn create_test_partition() -> PartitionEntry {
        PartitionEntry::new_with_params(
            true,                    // Bootable
            PartitionKind::Fat32Lba, // Type
            2048,                    // Start LBA
            204800,                  // Size in sectors (100MB)
        )
    }

    #[test]
    fn test_partition_entry_new() {
        let entry = PartitionEntry::new();
        assert!(!entry.is_valid());
        assert!(!entry.is_bootable());
        assert_eq!(entry.get_start_lba(), 0);
        assert_eq!(entry.get_size_sectors(), 0);
        assert_eq!(entry.get_partition_type(), PartitionKind::Empty);
    }

    #[test]
    fn test_partition_entry_new_with_params() {
        let entry = create_test_partition();
        assert!(entry.is_valid());
        assert!(entry.is_bootable());
        assert_eq!(entry.get_start_lba(), 2048);
        assert_eq!(entry.get_size_sectors(), 204800);
        assert_eq!(entry.get_partition_type(), PartitionKind::Fat32Lba);
    }

    #[test]
    fn test_partition_entry_bootable() {
        let mut entry = PartitionEntry::new();
        assert!(!entry.is_bootable());

        entry.set_bootable(true);
        assert!(entry.is_bootable());
        assert_eq!(entry.bootable, 0x80);

        entry.set_bootable(false);
        assert!(!entry.is_bootable());
        assert_eq!(entry.bootable, 0x00);
    }

    #[test]
    fn test_partition_entry_lba() {
        let mut entry = PartitionEntry::new();
        assert_eq!(entry.get_start_lba(), 0);

        entry.set_start_lba(12345);
        assert_eq!(entry.get_start_lba(), 12345);
    }

    #[test]
    fn test_partition_entry_size() {
        let mut entry = PartitionEntry::new();
        assert_eq!(entry.get_size_sectors(), 0);

        entry.set_size_sectors(67890);
        assert_eq!(entry.get_size_sectors(), 67890);
        assert_eq!(entry.get_size_bytes(), 67890 * 512);
    }

    #[test]
    fn test_partition_entry_type() {
        let mut entry = PartitionEntry::new();
        assert_eq!(entry.get_partition_type(), PartitionKind::Empty);

        entry.set_partition_type(PartitionKind::Linux);
        assert_eq!(entry.get_partition_type(), PartitionKind::Linux);
        assert_eq!(entry.partition_type, 0x83);
    }

    #[test]
    fn test_partition_entry_end_lba() {
        let entry = create_test_partition();
        assert_eq!(entry.get_end_lba(), 2048 + 204800 - 1);
    }

    #[test]
    fn test_partition_entry_overlaps() {
        let partition1 = PartitionEntry::new_with_params(false, PartitionKind::Fat32, 1000, 2000);
        let partition2 = PartitionEntry::new_with_params(false, PartitionKind::Linux, 2400, 1000);
        let partition3 =
            PartitionEntry::new_with_params(false, PartitionKind::LinuxSwap, 1500, 1000);

        // Partition1: 1000-2999, Partition2: 2400-3399, Partition3: 1500-2499
        assert!(partition1.overlaps_with(&partition3)); // 1000-2999 overlaps 1500-2499
        assert!(partition2.overlaps_with(&partition3)); // 2400-3399 overlaps 1500-2499 (overlap: 2400-2499)
        assert!(partition1.overlaps_with(&partition2)); // 1000-2999 overlaps 2400-3399 (overlap: 2400-2999)
    }

    #[test]
    fn test_partition_entry_no_overlap() {
        let partition1 = PartitionEntry::new_with_params(false, PartitionKind::Fat32, 1000, 1000);
        let partition2 = PartitionEntry::new_with_params(false, PartitionKind::Linux, 2000, 1000);

        // Partition1: 1000-1999, Partition2: 2000-2999
        assert!(!partition1.overlaps_with(&partition2));
        assert!(!partition2.overlaps_with(&partition1));
    }

    #[test]
    fn test_partition_entry_contains_lba() {
        let entry = create_test_partition();

        assert!(!entry.contains_lba(2047)); // Before start
        assert!(entry.contains_lba(2048)); // At start
        assert!(entry.contains_lba(100000)); // In middle
        assert!(entry.contains_lba(206847)); // At end (2048 + 204800 - 1)
        assert!(!entry.contains_lba(206848)); // After end
    }

    #[test]
    fn test_partition_entry_clear() {
        let mut entry = create_test_partition();
        assert!(entry.is_valid());

        entry.clear();
        assert!(!entry.is_valid());
        assert!(!entry.is_bootable());
        assert_eq!(entry.get_start_lba(), 0);
        assert_eq!(entry.get_size_sectors(), 0);
    }

    #[test]
    fn test_partition_entry_default() {
        let entry = PartitionEntry::default();
        assert!(!entry.is_valid());
        assert_eq!(entry.get_partition_type(), PartitionKind::Empty);
    }

    #[test]
    fn test_partition_entry_display() {
        let entry = create_test_partition();
        let display_string = format!("{entry}");

        assert!(display_string.contains("Type=0C"));
        assert!(display_string.contains("FAT32 LBA"));
        assert!(display_string.contains("Start_LBA=2048"));
        assert!(display_string.contains("Size=204800"));
        assert!(display_string.contains("Bootable=true"));

        let empty_entry = PartitionEntry::new();
        let empty_string = format!("{empty_entry}");
        assert!(empty_string.contains("Empty partition"));
    }

    #[test]
    fn test_partition_entry_size_bytes() {
        let entry = PartitionEntry::new_with_params(false, PartitionKind::Linux, 0, 2048);
        assert_eq!(entry.get_size_bytes(), 2048 * 512); // 1MB
    }

    #[test]
    fn test_partition_entry_validity() {
        // Valid partition must have non-zero type and size
        let valid = PartitionEntry::new_with_params(false, PartitionKind::Linux, 100, 200);
        assert!(valid.is_valid());

        // Zero size makes it invalid
        let zero_size = PartitionEntry::new_with_params(false, PartitionKind::Linux, 100, 0);
        assert!(!zero_size.is_valid());

        // Empty type makes it invalid
        let empty_type = PartitionEntry::new_with_params(false, PartitionKind::Empty, 100, 200);
        assert!(!empty_type.is_valid());
    }
}
