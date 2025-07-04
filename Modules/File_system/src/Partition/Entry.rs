//! Partition entry structures for MBR partition tables.
//!
//! This module provides the [`Partition_entry_type`] structure which represents
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
/// use File_system::*;
///
/// // Create a new bootable FAT32 partition
/// let partition = Partition_entry_type::New_with_params(
///     true,
///     Partition_type_type::Fat32_lba,
///     2048,
///     204800
/// );
///
/// assert!(partition.Is_bootable());
/// assert_eq!(partition.Get_start_lba(), 2048);
/// assert_eq!(partition.Get_size_sectors(), 204800);
/// ```
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Partition_entry_type {
    /// Boot indicator (0x80 = bootable, 0x00 = non-bootable)
    pub Bootable: u8,
    /// Starting head
    pub Start_head: u8,
    /// Starting sector (bits 5-0) and cylinder high bits (bits 7-6)
    pub Start_sector: u8,
    /// Starting cylinder (low 8 bits)
    pub Start_cylinder: u8,
    /// Partition type ID
    pub Partition_type: u8,
    /// Ending head
    pub End_head: u8,
    /// Ending sector (bits 5-0) and cylinder high bits (bits 7-6)
    pub End_sector: u8,
    /// Ending cylinder (low 8 bits)
    pub End_cylinder: u8,
    /// Starting LBA (Logical Block Address)
    pub Start_lba: u32,
    /// Size in sectors
    pub Size_sectors: u32,
}

impl Partition_entry_type {
    /// Create a new empty (invalid) partition entry.
    ///
    /// All fields are initialized to zero, making this an invalid partition entry
    /// that will not be recognized by the MBR parser.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate alloc;
    /// use File_system::*;
    ///
    /// let partition = Partition_entry_type::New();
    /// assert!(!partition.Is_valid());
    /// assert!(!partition.Is_bootable());
    /// ```
    pub fn New() -> Self {
        Self {
            Bootable: 0,
            Start_head: 0,
            Start_sector: 0,
            Start_cylinder: 0,
            Partition_type: 0,
            End_head: 0,
            End_sector: 0,
            End_cylinder: 0,
            Start_lba: 0,
            Size_sectors: 0,
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
    /// use File_system::*;
    ///
    /// // Create a 100MB FAT32 partition starting at sector 2048
    /// let partition = Partition_entry_type::New_with_params(
    ///     true,
    ///     Partition_type_type::Fat32_lba,
    ///     2048,
    ///     204800
    /// );
    ///
    /// assert!(partition.Is_valid());
    /// assert!(partition.Is_bootable());
    /// ```
    pub fn New_with_params(
        Bootable: bool,
        Partition_type: crate::Partition_type_type,
        Start_lba: u32,
        Size_sectors: u32,
    ) -> Self {
        let mut Entry = Self::New();
        Entry.Bootable = if Bootable { 0x80 } else { 0x00 };
        Entry.Set_partition_type(Partition_type);
        Entry.Start_lba = Start_lba.to_le();
        Entry.Size_sectors = Size_sectors.to_le();
        Entry
    }

    /// Check if this partition entry is valid (non-zero)
    pub fn Is_valid(&self) -> bool {
        self.Partition_type != 0 && self.Size_sectors > 0
    }

    /// Check if this partition is bootable
    pub fn Is_bootable(&self) -> bool {
        self.Bootable == 0x80
    }

    /// Set the bootable flag
    pub fn Set_bootable(&mut self, Bootable: bool) {
        self.Bootable = if Bootable { 0x80 } else { 0x00 };
    }

    /// Get the starting LBA of this partition
    pub fn Get_start_lba(&self) -> u32 {
        u32::from_le(self.Start_lba)
    }

    /// Set the starting LBA of this partition
    pub fn Set_start_lba(&mut self, Start_lba: u32) {
        self.Start_lba = Start_lba.to_le();
    }

    /// Get the size in sectors of this partition
    pub fn Get_size_sectors(&self) -> u32 {
        u32::from_le(self.Size_sectors)
    }

    /// Set the size in sectors of this partition
    pub fn Set_size_sectors(&mut self, Size_sectors: u32) {
        self.Size_sectors = Size_sectors.to_le();
    }

    /// Get the partition type as an enum
    pub fn Get_partition_type(&self) -> crate::Partition_type_type {
        crate::Partition_type_type::From_u8(self.Partition_type)
    }

    /// Set the partition type from an enum
    pub fn Set_partition_type(&mut self, Partition_type: crate::Partition_type_type) {
        self.Partition_type = Partition_type.To_u8();
    }

    /// Get the partition type as a human-readable string
    pub fn Get_partition_type_name(&self) -> &'static str {
        self.Get_partition_type().Get_name()
    }

    /// Get the end LBA of this partition (start + size - 1)
    pub fn Get_end_lba(&self) -> u32 {
        self.Get_start_lba() + self.Get_size_sectors() - 1
    }

    /// Get the size in bytes of this partition
    pub fn Get_size_bytes(&self) -> u64 {
        self.Get_size_sectors() as u64 * 512
    }

    /// Check if this partition overlaps with another partition
    pub fn Overlaps_with(&self, Other: &Self) -> bool {
        if !self.Is_valid() || !Other.Is_valid() {
            return false;
        }

        let Self_start = self.Get_start_lba();
        let Self_end = self.Get_end_lba();
        let Other_start = Other.Get_start_lba();
        let Other_end = Other.Get_end_lba();

        !(Self_end < Other_start || Other_end < Self_start)
    }

    /// Check if a given LBA is within this partition
    pub fn Contains_lba(&self, Lba: u32) -> bool {
        if !self.Is_valid() {
            return false;
        }

        let Start = self.Get_start_lba();
        let End = self.Get_end_lba();
        Lba >= Start && Lba <= End
    }

    /// Clear the partition entry (make it empty)
    pub fn Clear(&mut self) {
        *self = Self::New();
    }
}

impl Default for Partition_entry_type {
    fn default() -> Self {
        Self::New()
    }
}

impl fmt::Display for Partition_entry_type {
    fn fmt(&self, Formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.Is_valid() {
            write!(Formatter, "Empty partition")
        } else {
            write!(
                Formatter,
                "Partition: Type={:02X} ({}), Start_LBA={}, Size={} sectors ({} MB), Bootable={}",
                self.Partition_type,
                self.Get_partition_type_name(),
                self.Get_start_lba(),
                self.Get_size_sectors(),
                self.Get_size_bytes() / (1024 * 1024),
                self.Is_bootable()
            )
        }
    }
}

#[cfg(test)]
mod Tests {
    use super::Partition_entry_type;
    use crate::Partition_type_type;
    use alloc::format;

    fn Create_test_partition() -> Partition_entry_type {
        Partition_entry_type::New_with_params(
            true,                           // Bootable
            Partition_type_type::Fat32_lba, // Type
            2048,                           // Start LBA
            204800,                         // Size in sectors (100MB)
        )
    }

    #[test]
    fn Test_partition_entry_new() {
        let Entry = Partition_entry_type::New();
        assert!(!Entry.Is_valid());
        assert!(!Entry.Is_bootable());
        assert_eq!(Entry.Get_start_lba(), 0);
        assert_eq!(Entry.Get_size_sectors(), 0);
        assert_eq!(Entry.Get_partition_type(), Partition_type_type::Empty);
    }

    #[test]
    fn Test_partition_entry_new_with_params() {
        let Entry = Create_test_partition();
        assert!(Entry.Is_valid());
        assert!(Entry.Is_bootable());
        assert_eq!(Entry.Get_start_lba(), 2048);
        assert_eq!(Entry.Get_size_sectors(), 204800);
        assert_eq!(Entry.Get_partition_type(), Partition_type_type::Fat32_lba);
    }

    #[test]
    fn Test_partition_entry_bootable() {
        let mut Entry = Partition_entry_type::New();
        assert!(!Entry.Is_bootable());

        Entry.Set_bootable(true);
        assert!(Entry.Is_bootable());
        assert_eq!(Entry.Bootable, 0x80);

        Entry.Set_bootable(false);
        assert!(!Entry.Is_bootable());
        assert_eq!(Entry.Bootable, 0x00);
    }

    #[test]
    fn Test_partition_entry_lba() {
        let mut Entry = Partition_entry_type::New();
        assert_eq!(Entry.Get_start_lba(), 0);

        Entry.Set_start_lba(12345);
        assert_eq!(Entry.Get_start_lba(), 12345);
    }

    #[test]
    fn Test_partition_entry_size() {
        let mut Entry = Partition_entry_type::New();
        assert_eq!(Entry.Get_size_sectors(), 0);

        Entry.Set_size_sectors(67890);
        assert_eq!(Entry.Get_size_sectors(), 67890);
        assert_eq!(Entry.Get_size_bytes(), 67890 * 512);
    }

    #[test]
    fn Test_partition_entry_type() {
        let mut Entry = Partition_entry_type::New();
        assert_eq!(Entry.Get_partition_type(), Partition_type_type::Empty);

        Entry.Set_partition_type(Partition_type_type::Linux);
        assert_eq!(Entry.Get_partition_type(), Partition_type_type::Linux);
        assert_eq!(Entry.Partition_type, 0x83);
    }

    #[test]
    fn Test_partition_entry_end_lba() {
        let Entry = Create_test_partition();
        assert_eq!(Entry.Get_end_lba(), 2048 + 204800 - 1);
    }

    #[test]
    fn Test_partition_entry_overlaps() {
        let Partition1 =
            Partition_entry_type::New_with_params(false, Partition_type_type::Fat32, 1000, 2000);
        let Partition2 =
            Partition_entry_type::New_with_params(false, Partition_type_type::Linux, 2400, 1000);
        let Partition3 = Partition_entry_type::New_with_params(
            false,
            Partition_type_type::Linux_swap,
            1500,
            1000,
        );

        // Partition1: 1000-2999, Partition2: 2400-3399, Partition3: 1500-2499
        assert!(Partition1.Overlaps_with(&Partition3)); // 1000-2999 overlaps 1500-2499
        assert!(Partition2.Overlaps_with(&Partition3)); // 2400-3399 overlaps 1500-2499 (overlap: 2400-2499)
        assert!(Partition1.Overlaps_with(&Partition2)); // 1000-2999 overlaps 2400-3399 (overlap: 2400-2999)
    }

    #[test]
    fn Test_partition_entry_no_overlap() {
        let Partition1 =
            Partition_entry_type::New_with_params(false, Partition_type_type::Fat32, 1000, 1000);
        let Partition2 =
            Partition_entry_type::New_with_params(false, Partition_type_type::Linux, 2000, 1000);

        // Partition1: 1000-1999, Partition2: 2000-2999
        assert!(!Partition1.Overlaps_with(&Partition2));
        assert!(!Partition2.Overlaps_with(&Partition1));
    }

    #[test]
    fn Test_partition_entry_contains_lba() {
        let Entry = Create_test_partition();

        assert!(!Entry.Contains_lba(2047)); // Before start
        assert!(Entry.Contains_lba(2048)); // At start
        assert!(Entry.Contains_lba(100000)); // In middle
        assert!(Entry.Contains_lba(206847)); // At end (2048 + 204800 - 1)
        assert!(!Entry.Contains_lba(206848)); // After end
    }

    #[test]
    fn Test_partition_entry_clear() {
        let mut Entry = Create_test_partition();
        assert!(Entry.Is_valid());

        Entry.Clear();
        assert!(!Entry.Is_valid());
        assert!(!Entry.Is_bootable());
        assert_eq!(Entry.Get_start_lba(), 0);
        assert_eq!(Entry.Get_size_sectors(), 0);
    }

    #[test]
    fn Test_partition_entry_default() {
        let Entry = Partition_entry_type::default();
        assert!(!Entry.Is_valid());
        assert_eq!(Entry.Get_partition_type(), Partition_type_type::Empty);
    }

    #[test]
    fn Test_partition_entry_display() {
        let Entry = Create_test_partition();
        let Display_string = format!("{Entry}");

        assert!(Display_string.contains("Type=0C"));
        assert!(Display_string.contains("FAT32 LBA"));
        assert!(Display_string.contains("Start_LBA=2048"));
        assert!(Display_string.contains("Size=204800"));
        assert!(Display_string.contains("Bootable=true"));

        let Empty_entry = Partition_entry_type::New();
        let Empty_string = format!("{Empty_entry}");
        assert!(Empty_string.contains("Empty partition"));
    }

    #[test]
    fn Test_partition_entry_size_bytes() {
        let Entry =
            Partition_entry_type::New_with_params(false, Partition_type_type::Linux, 0, 2048);
        assert_eq!(Entry.Get_size_bytes(), 2048 * 512); // 1MB
    }

    #[test]
    fn Test_partition_entry_validity() {
        // Valid partition must have non-zero type and size
        let Valid =
            Partition_entry_type::New_with_params(false, Partition_type_type::Linux, 100, 200);
        assert!(Valid.Is_valid());

        // Zero size makes it invalid
        let Zero_size =
            Partition_entry_type::New_with_params(false, Partition_type_type::Linux, 100, 0);
        assert!(!Zero_size.Is_valid());

        // Empty type makes it invalid
        let Empty_type =
            Partition_entry_type::New_with_params(false, Partition_type_type::Empty, 100, 200);
        assert!(!Empty_type.Is_valid());
    }
}
