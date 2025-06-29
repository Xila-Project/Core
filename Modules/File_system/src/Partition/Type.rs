use core::fmt;

/// MBR Partition type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Partition_type_type {
    Empty = 0x00,
    Fat12 = 0x01,
    Fat16_small = 0x04,
    Extended = 0x05,
    Fat16 = 0x06,
    Ntfs_exfat = 0x07,
    Fat32 = 0x0B,
    Fat32_lba = 0x0C,
    Fat16_lba = 0x0E,
    Extended_lba = 0x0F,
    Hidden_fat12 = 0x11,
    Hidden_fat16_small = 0x14,
    Hidden_fat16 = 0x16,
    Hidden_ntfs_exfat = 0x17,
    Hidden_fat32 = 0x1B,
    Hidden_fat32_lba = 0x1C,
    Hidden_fat16_lba = 0x1E,
    Linux_swap = 0x82,
    Linux = 0x83,
    Linux_lvm = 0x8E,
    Gpt_protective = 0xEE,
    Efi_system = 0xEF,
    Unknown(u8),
}

impl Partition_type_type {
    /// Create a partition type from a raw u8 value
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
            Partition_type_type::Unknown(Value) => *Value,
        }
    }

    /// Get the human-readable name of the partition type
    pub fn Get_name(&self) -> &'static str {
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
            Partition_type_type::Unknown(_) => "Unknown",
        }
    }

    /// Check if this partition type is a FAT filesystem
    pub fn Is_fat(&self) -> bool {
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
    pub fn Is_hidden(&self) -> bool {
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
    pub fn Is_extended(&self) -> bool {
        matches!(
            self,
            Partition_type_type::Extended | Partition_type_type::Extended_lba
        )
    }

    /// Check if this partition type is Linux-related
    pub fn Is_linux(&self) -> bool {
        matches!(
            self,
            Partition_type_type::Linux
                | Partition_type_type::Linux_swap
                | Partition_type_type::Linux_lvm
        )
    }
}

impl fmt::Display for Partition_type_type {
    fn fmt(&self, Formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Partition_type_type::Unknown(Value) => write!(Formatter, "Unknown (0x{Value:02X})"),
            _ => write!(Formatter, "{} (0x{:02X})", self.Get_name(), self.To_u8()),
        }
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use alloc::{format, vec};

    #[test]
    fn Test_partition_type_from_u8() {
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
    fn Test_partition_type_to_u8() {
        assert_eq!(Partition_type_type::Empty.To_u8(), 0x00);
        assert_eq!(Partition_type_type::Fat32_lba.To_u8(), 0x0C);
        assert_eq!(Partition_type_type::Linux.To_u8(), 0x83);
        assert_eq!(Partition_type_type::Gpt_protective.To_u8(), 0xEE);
        assert_eq!(Partition_type_type::Unknown(0xFF).To_u8(), 0xFF);
    }

    #[test]
    fn Test_partition_type_round_trip() {
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
    fn Test_partition_type_properties() {
        // Test FAT detection
        assert!(Partition_type_type::Fat12.Is_fat());
        assert!(Partition_type_type::Fat16.Is_fat());
        assert!(Partition_type_type::Fat32.Is_fat());
        assert!(Partition_type_type::Fat32_lba.Is_fat());
        assert!(Partition_type_type::Hidden_fat32.Is_fat());
        assert!(!Partition_type_type::Linux.Is_fat());
        assert!(!Partition_type_type::Ntfs_exfat.Is_fat());

        // Test hidden detection
        assert!(Partition_type_type::Hidden_fat12.Is_hidden());
        assert!(Partition_type_type::Hidden_fat32.Is_hidden());
        assert!(Partition_type_type::Hidden_ntfs_exfat.Is_hidden());
        assert!(!Partition_type_type::Fat32.Is_hidden());
        assert!(!Partition_type_type::Linux.Is_hidden());

        // Test extended detection
        assert!(Partition_type_type::Extended.Is_extended());
        assert!(Partition_type_type::Extended_lba.Is_extended());
        assert!(!Partition_type_type::Fat32.Is_extended());
        assert!(!Partition_type_type::Linux.Is_extended());

        // Test Linux detection
        assert!(Partition_type_type::Linux.Is_linux());
        assert!(Partition_type_type::Linux_swap.Is_linux());
        assert!(Partition_type_type::Linux_lvm.Is_linux());
        assert!(!Partition_type_type::Fat32.Is_linux());
        assert!(!Partition_type_type::Ntfs_exfat.Is_linux());
    }

    #[test]
    fn Test_partition_type_names() {
        assert_eq!(Partition_type_type::Empty.Get_name(), "Empty");
        assert_eq!(Partition_type_type::Fat32_lba.Get_name(), "FAT32 LBA");
        assert_eq!(Partition_type_type::Linux.Get_name(), "Linux");
        assert_eq!(
            Partition_type_type::Gpt_protective.Get_name(),
            "GPT protective"
        );
        assert_eq!(Partition_type_type::Unknown(0x42).Get_name(), "Unknown");
    }

    #[test]
    fn Test_partition_type_display() {
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
