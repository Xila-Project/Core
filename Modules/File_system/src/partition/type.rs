//! Partition type definitions for MBR partition tables.
//!
//! This module provides the [`Partition_type_type`] enumeration which defines
//! all standard partition types used in MBR partition tables. Each partition
//! type corresponds to a specific file system or partition purpose.

use core::fmt;

/// MBR partition type enumeration with comprehensive type definitions.
///
/// This enum represents the partition type field in MBR partition entries.
/// Each variant corresponds to a specific file system type or partition purpose
/// as defined by the standard PC partition type IDs.
///
/// # Standard Partition Types
///
/// ## FAT File Systems
/// - [`Fat12`] - FAT12 file system (floppy disks, small partitions)
/// - [`Fat16`] - FAT16 file system
/// - [`Fat16_small`] - FAT16 for partitions < 32MB
/// - [`Fat32`] - FAT32 file system
/// - [`Fat32_lba`] - FAT32 with LBA addressing (recommended)
///
/// ## Extended Partitions
/// - [`Extended`] - Extended partition (CHS addressing)
/// - [`Extended_lba`] - Extended partition (LBA addressing)
///
/// ## Linux File Systems
/// - [`Linux`] - Linux native partition (typically ext2/3/4)
/// - [`Linux_swap`] - Linux swap partition
/// - [`Linux_lvm`] - Linux LVM (Logical Volume Manager)
///
/// ## Other File Systems
/// - [`Ntfs_exfat`] - NTFS or exFAT file system
/// - [`Efi_system`] - EFI System Partition
/// - [`Gpt_protective`] - GPT protective partition
///
/// # Examples
///
/// ```rust
/// use file_system::*;
///
/// let partition_type = Partition_type_type::Fat32_lba;
/// assert!(partition_type.is_fat());
/// assert!(!partition_type.is_extended());
///
/// let linux_type = Partition_type_type::From_u8(0x83);
/// assert_eq!(linux_type, Partition_type_type::Linux);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Partition_type_type {
    /// Empty/unused partition slot
    Empty = 0x00,
    /// FAT12 file system
    Fat12 = 0x01,
    /// FAT16 file system (small partitions < 32MB)
    Fat16_small = 0x04,
    /// Extended partition (CHS addressing)
    Extended = 0x05,
    /// FAT16 file system
    Fat16 = 0x06,
    /// NTFS or exFAT file system
    Ntfs_exfat = 0x07,
    /// FAT32 file system (CHS addressing)
    Fat32 = 0x0B,
    /// FAT32 file system (LBA addressing - recommended)
    Fat32_lba = 0x0C,
    /// FAT16 file system (LBA addressing)
    Fat16_lba = 0x0E,
    /// Extended partition (LBA addressing)
    Extended_lba = 0x0F,
    /// Hidden FAT12 partition
    Hidden_fat12 = 0x11,
    /// Hidden FAT16 partition (small)
    Hidden_fat16_small = 0x14,
    /// Hidden FAT16 partition
    Hidden_fat16 = 0x16,
    /// Hidden NTFS/exFAT partition
    Hidden_ntfs_exfat = 0x17,
    /// Hidden FAT32 partition
    Hidden_fat32 = 0x1B,
    /// Hidden FAT32 partition (LBA addressing)
    Hidden_fat32_lba = 0x1C,
    /// Hidden FAT16 partition (LBA addressing)
    Hidden_fat16_lba = 0x1E,
    /// Linux swap partition
    Linux_swap = 0x82,
    /// Linux native partition (typically ext2/3/4)
    Linux = 0x83,
    /// Linux LVM (Logical Volume Manager)
    Linux_lvm = 0x8E,
    /// GPT protective partition (indicates GPT, not MBR)
    Gpt_protective = 0xEE,
    /// EFI System Partition
    Efi_system = 0xEF,
    /// Xila
    Xila = 0xDA,
    /// Unknown or custom partition type
    Unknown(u8),
}

impl Partition_type_type {
    /// Create a partition type from a raw u8 value.
    ///
    /// This function maps standard partition type IDs to their corresponding
    /// enum variants. Unknown types are wrapped in [`Partition_type_type::Unknown`].
    ///
    /// # Arguments
    ///
    /// * `Value` - The raw partition type ID from the MBR partition entry
    ///
    /// # Returns
    ///
    /// The corresponding partition type enum variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_system::*;
    ///
    /// assert_eq!(Partition_type_type::From_u8(0x0C), Partition_type_type::Fat32_lba);
    /// assert_eq!(Partition_type_type::From_u8(0x83), Partition_type_type::Linux);
    ///
    /// // Unknown types are preserved
    /// if let Partition_type_type::Unknown(id) = Partition_type_type::From_u8(0xFF) {
    ///     assert_eq!(id, 0xFF);
    /// }
    /// ```
    pub fn From_u8(Value: u8) -> Self {
        match Value {
            0x00 => Partition_type_type::Empty,
            0x01 => Partition_type_type::Fat12,
            0x04 => Partition_type_type::Fat16_small,
            0x05 => Partition_type_type::Extended,
            0x06 => Partition_type_type::Fat16,
            0x07 => Partition_type_type::Ntfs_exfat,
            0x0B => Partition_type_type::Fat32,
            0x0C => Partition_type_type::Fat32_lba,
            0x0E => Partition_type_type::Fat16_lba,
            0x0F => Partition_type_type::Extended_lba,
            0x11 => Partition_type_type::Hidden_fat12,
            0x14 => Partition_type_type::Hidden_fat16_small,
            0x16 => Partition_type_type::Hidden_fat16,
            0x17 => Partition_type_type::Hidden_ntfs_exfat,
            0x1B => Partition_type_type::Hidden_fat32,
            0x1C => Partition_type_type::Hidden_fat32_lba,
            0x1E => Partition_type_type::Hidden_fat16_lba,
            0x82 => Partition_type_type::Linux_swap,
            0x83 => Partition_type_type::Linux,
            0x8E => Partition_type_type::Linux_lvm,
            0xEE => Partition_type_type::Gpt_protective,
            0xEF => Partition_type_type::Efi_system,
            0xDA => Partition_type_type::Xila,
            _ => Partition_type_type::Unknown(Value),
        }
    }

    /// Convert the partition type to its raw u8 value
    pub fn To_u8(&self) -> u8 {
        match self {
            Partition_type_type::Empty => 0x00,
            Partition_type_type::Fat12 => 0x01,
            Partition_type_type::Fat16_small => 0x04,
            Partition_type_type::Extended => 0x05,
            Partition_type_type::Fat16 => 0x06,
            Partition_type_type::Ntfs_exfat => 0x07,
            Partition_type_type::Fat32 => 0x0B,
            Partition_type_type::Fat32_lba => 0x0C,
            Partition_type_type::Fat16_lba => 0x0E,
            Partition_type_type::Extended_lba => 0x0F,
            Partition_type_type::Hidden_fat12 => 0x11,
            Partition_type_type::Hidden_fat16_small => 0x14,
            Partition_type_type::Hidden_fat16 => 0x16,
            Partition_type_type::Hidden_ntfs_exfat => 0x17,
            Partition_type_type::Hidden_fat32 => 0x1B,
            Partition_type_type::Hidden_fat32_lba => 0x1C,
            Partition_type_type::Hidden_fat16_lba => 0x1E,
            Partition_type_type::Linux_swap => 0x82,
            Partition_type_type::Linux => 0x83,
            Partition_type_type::Linux_lvm => 0x8E,
            Partition_type_type::Gpt_protective => 0xEE,
            Partition_type_type::Efi_system => 0xEF,
            Partition_type_type::Xila => 0xDA,
            Partition_type_type::Unknown(value) => *value,
        }
    }

    /// Get the human-readable name of the partition type
    pub fn get_name(&self) -> &'static str {
        match self {
            Partition_type_type::Empty => "Empty",
            Partition_type_type::Fat12 => "FAT12",
            Partition_type_type::Fat16_small => "FAT16 <32M",
            Partition_type_type::Extended => "Extended",
            Partition_type_type::Fat16 => "FAT16",
            Partition_type_type::Ntfs_exfat => "NTFS/exFAT",
            Partition_type_type::Fat32 => "FAT32",
            Partition_type_type::Fat32_lba => "FAT32 LBA",
            Partition_type_type::Fat16_lba => "FAT16 LBA",
            Partition_type_type::Extended_lba => "Extended LBA",
            Partition_type_type::Hidden_fat12 => "Hidden FAT12",
            Partition_type_type::Hidden_fat16_small => "Hidden FAT16 <32M",
            Partition_type_type::Hidden_fat16 => "Hidden FAT16",
            Partition_type_type::Hidden_ntfs_exfat => "Hidden NTFS/exFAT",
            Partition_type_type::Hidden_fat32 => "Hidden FAT32",
            Partition_type_type::Hidden_fat32_lba => "Hidden FAT32 LBA",
            Partition_type_type::Hidden_fat16_lba => "Hidden FAT16 LBA",
            Partition_type_type::Linux_swap => "Linux swap",
            Partition_type_type::Linux => "Linux",
            Partition_type_type::Linux_lvm => "Linux LVM",
            Partition_type_type::Gpt_protective => "GPT protective",
            Partition_type_type::Efi_system => "EFI System",
            Partition_type_type::Xila => "Xila",
            Partition_type_type::Unknown(_) => "Unknown",
        }
    }

    /// Check if this partition type is a FAT filesystem
    pub fn is_fat(&self) -> bool {
        matches!(
            self,
            Partition_type_type::Fat12
                | Partition_type_type::Fat16_small
                | Partition_type_type::Fat16
                | Partition_type_type::Fat32
                | Partition_type_type::Fat32_lba
                | Partition_type_type::Fat16_lba
                | Partition_type_type::Hidden_fat12
                | Partition_type_type::Hidden_fat16_small
                | Partition_type_type::Hidden_fat16
                | Partition_type_type::Hidden_fat32
                | Partition_type_type::Hidden_fat32_lba
                | Partition_type_type::Hidden_fat16_lba
        )
    }

    /// Check if this partition type is hidden
    pub fn is_hidden(&self) -> bool {
        matches!(
            self,
            Partition_type_type::Hidden_fat12
                | Partition_type_type::Hidden_fat16_small
                | Partition_type_type::Hidden_fat16
                | Partition_type_type::Hidden_ntfs_exfat
                | Partition_type_type::Hidden_fat32
                | Partition_type_type::Hidden_fat32_lba
                | Partition_type_type::Hidden_fat16_lba
        )
    }

    /// Check if this partition type is an extended partition
    pub fn is_extended(&self) -> bool {
        matches!(
            self,
            Partition_type_type::Extended | Partition_type_type::Extended_lba
        )
    }

    /// Check if this partition type is Linux-related
    pub fn is_linux(&self) -> bool {
        matches!(
            self,
            Partition_type_type::Linux
                | Partition_type_type::Linux_swap
                | Partition_type_type::Linux_lvm
        )
    }
}

impl fmt::Display for Partition_type_type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Partition_type_type::Unknown(Value) => write!(formatter, "Unknown (0x{Value:02X})"),
            _ => write!(formatter, "{} (0x{:02X})", self.get_name(), self.To_u8()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, vec};

    #[test]
    fn test_partition_type_from_u8() {
        assert_eq!(
            Partition_type_type::From_u8(0x00),
            Partition_type_type::Empty
        );
        assert_eq!(
            Partition_type_type::From_u8(0x0C),
            Partition_type_type::Fat32_lba
        );
        assert_eq!(
            Partition_type_type::From_u8(0x83),
            Partition_type_type::Linux
        );
        assert_eq!(
            Partition_type_type::From_u8(0xEE),
            Partition_type_type::Gpt_protective
        );
        assert_eq!(
            Partition_type_type::From_u8(0xFF),
            Partition_type_type::Unknown(0xFF)
        );
    }

    #[test]
    fn test_partition_type_to_u8() {
        assert_eq!(Partition_type_type::Empty.To_u8(), 0x00);
        assert_eq!(Partition_type_type::Fat32_lba.To_u8(), 0x0C);
        assert_eq!(Partition_type_type::Linux.To_u8(), 0x83);
        assert_eq!(Partition_type_type::Gpt_protective.To_u8(), 0xEE);
        assert_eq!(Partition_type_type::Unknown(0xFF).To_u8(), 0xFF);
    }

    #[test]
    fn test_partition_type_round_trip() {
        let Types = vec![
            0x00, 0x01, 0x04, 0x05, 0x06, 0x07, 0x0B, 0x0C, 0x0E, 0x0F, 0x11, 0x14, 0x16, 0x17,
            0x1B, 0x1C, 0x1E, 0x82, 0x83, 0x8E, 0xEE, 0xEF, 0xFF, 0x42, 0x99,
        ];

        for Type_value in Types {
            let Partition_type = Partition_type_type::From_u8(Type_value);
            assert_eq!(Partition_type.To_u8(), Type_value);
        }
    }

    #[test]
    fn test_partition_type_properties() {
        // Test FAT detection
        assert!(Partition_type_type::Fat12.is_fat());
        assert!(Partition_type_type::Fat16.is_fat());
        assert!(Partition_type_type::Fat32.is_fat());
        assert!(Partition_type_type::Fat32_lba.is_fat());
        assert!(Partition_type_type::Hidden_fat32.is_fat());
        assert!(!Partition_type_type::Linux.is_fat());
        assert!(!Partition_type_type::Ntfs_exfat.is_fat());

        // Test hidden detection
        assert!(Partition_type_type::Hidden_fat12.is_hidden());
        assert!(Partition_type_type::Hidden_fat32.is_hidden());
        assert!(Partition_type_type::Hidden_ntfs_exfat.is_hidden());
        assert!(!Partition_type_type::Fat32.is_hidden());
        assert!(!Partition_type_type::Linux.is_hidden());

        // Test extended detection
        assert!(Partition_type_type::Extended.is_extended());
        assert!(Partition_type_type::Extended_lba.is_extended());
        assert!(!Partition_type_type::Fat32.is_extended());
        assert!(!Partition_type_type::Linux.is_extended());

        // Test Linux detection
        assert!(Partition_type_type::Linux.is_linux());
        assert!(Partition_type_type::Linux_swap.is_linux());
        assert!(Partition_type_type::Linux_lvm.is_linux());
        assert!(!Partition_type_type::Fat32.is_linux());
        assert!(!Partition_type_type::Ntfs_exfat.is_linux());
    }

    #[test]
    fn test_partition_type_names() {
        assert_eq!(Partition_type_type::Empty.get_name(), "Empty");
        assert_eq!(Partition_type_type::Fat32_lba.get_name(), "FAT32 LBA");
        assert_eq!(Partition_type_type::Linux.get_name(), "Linux");
        assert_eq!(
            Partition_type_type::Gpt_protective.get_name(),
            "GPT protective"
        );
        assert_eq!(Partition_type_type::Unknown(0x42).get_name(), "Unknown");
    }

    #[test]
    fn test_partition_type_display() {
        let Fat32_variant = Partition_type_type::Fat32_lba;
        let Display_string = format!("{Fat32_variant}");
        assert!(Display_string.contains("FAT32 LBA"));
        assert!(Display_string.contains("0x0C"));

        let Unknown = Partition_type_type::Unknown(0x42);
        let Unknown_string = format!("{Unknown}");
        assert!(Unknown_string.contains("Unknown"));
        assert!(Unknown_string.contains("0x42"));
    }
}
