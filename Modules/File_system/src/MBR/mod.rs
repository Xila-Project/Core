//! Master Boot Record (MBR) module
//!
//! This module provides functionality for working with MBR (Master Boot Record)
//! partition tables, which are used in traditional BIOS-based systems.
//!
//! # Features
//!
//! - Parse and validate MBR structures from raw bytes or devices
//! - Create and modify MBR partition tables
//! - Work with individual partition entries
//! - Create partition devices for accessing individual partitions
//! - Utility functions for common MBR operations
//! - Type-safe partition type enumeration
//!
//! # Examples
//!
//! ```rust
//! extern crate alloc;
//!
//! use File_system::*;
//!
//! let device = Create_device!(Memory_device_type::<512>::New(512));
//!
//! // Read MBR from a device
//! let mbr = MBR_type::Read_from_device(&device).unwrap();
//!
//! // Display MBR information
//! println!("{}", mbr);
//!
//! // Get all valid partitions
//! let partitions = mbr.Get_valid_partitions();
//!
//! // Create a partition device
//! if let Some(partition) = partitions.first() {
//!     let partition_device = Create_partition_device(device.clone(), partition).unwrap();
//! }
//! ```

use alloc::vec::Vec;
use core::fmt;
mod Utilities;
pub use Utilities::*;

use crate::{Device_type, Error_type, MBR_partition_entry, Partition_type, Result_type};

/// Master Boot Record structure (512 bytes)
#[derive(Debug, Clone)]
pub struct MBR_type {
    /// Bootstrap code (440 bytes)
    pub Bootstrap_code: [u8; 440],
    /// Optional disk signature (4 bytes)
    pub Disk_signature: [u8; 4],
    /// Reserved (usually 0x0000)
    pub Reserved: [u8; 2],
    /// Partition table (4 entries × 16 bytes = 64 bytes)
    pub Partitions: [MBR_partition_entry; 4],
    /// Boot signature (0x55AA)
    pub Boot_signature: [u8; 2],
}

impl MBR_type {
    /// MBR signature bytes
    pub const SIGNATURE: [u8; 2] = [0x55, 0xAA];

    /// Size of MBR in bytes
    pub const SIZE: usize = 512;

    /// Maximum number of primary partitions in MBR
    pub const MAX_PARTITIONS: usize = 4;

    /// Create a new empty MBR
    pub fn New() -> Self {
        Self {
            Bootstrap_code: [0; 440],
            Disk_signature: [0; 4],
            Reserved: [0; 2],
            Partitions: [MBR_partition_entry::New(); 4],
            Boot_signature: Self::SIGNATURE,
        }
    }

    /// Create a new MBR with a specific disk signature
    pub fn New_with_signature(Disk_signature: u32) -> Self {
        let mut Mbr = Self::New();
        Mbr.Set_disk_signature(Disk_signature);
        Mbr
    }

    /// Parse MBR from raw bytes
    pub fn From_bytes(Data: &[u8]) -> Result_type<Self> {
        if Data.len() < Self::SIZE {
            return Err(Error_type::Invalid_parameter);
        }

        // Check MBR signature
        if Data[510] != Self::SIGNATURE[0] || Data[511] != Self::SIGNATURE[1] {
            return Err(Error_type::Corrupted);
        }

        let mut Mbr = MBR_type {
            Bootstrap_code: [0; 440],
            Disk_signature: [0; 4],
            Reserved: [0; 2],
            Partitions: [MBR_partition_entry::New(); 4],
            Boot_signature: [0; 2],
        };

        // Copy bootstrap code
        Mbr.Bootstrap_code.copy_from_slice(&Data[0..440]);

        // Copy disk signature
        Mbr.Disk_signature.copy_from_slice(&Data[440..444]);

        // Copy reserved bytes
        Mbr.Reserved.copy_from_slice(&Data[444..446]);

        // Parse partition entries
        for (I, Partition) in Mbr.Partitions.iter_mut().enumerate() {
            let Offset = 446 + (I * 16);
            let Partition_data = &Data[Offset..Offset + 16];

            Partition.Bootable = Partition_data[0];
            Partition.Start_head = Partition_data[1];
            Partition.Start_sector = Partition_data[2];
            Partition.Start_cylinder = Partition_data[3];
            Partition.Partition_type = Partition_data[4];
            Partition.End_head = Partition_data[5];
            Partition.End_sector = Partition_data[6];
            Partition.End_cylinder = Partition_data[7];
            Partition.Start_lba = u32::from_le_bytes([
                Partition_data[8],
                Partition_data[9],
                Partition_data[10],
                Partition_data[11],
            ]);
            Partition.Size_sectors = u32::from_le_bytes([
                Partition_data[12],
                Partition_data[13],
                Partition_data[14],
                Partition_data[15],
            ]);
        }

        // Copy boot signature
        Mbr.Boot_signature.copy_from_slice(&Data[510..512]);

        Ok(Mbr)
    }

    /// Read and parse MBR from a device
    pub fn Read_from_device(Device: &Device_type) -> Result_type<Self> {
        // Read the first 512 bytes (MBR sector)
        let mut Buffer = [0u8; Self::SIZE];

        // Set position to the beginning of the device
        Device.Set_position(&crate::Position_type::Start(0))?;

        // Read MBR data
        let Bytes_read = Device.Read(&mut Buffer)?;

        if Bytes_read.As_u64() < Self::SIZE as u64 {
            return Err(Error_type::Input_output);
        }

        Self::From_bytes(&Buffer)
    }

    /// Write MBR to a device
    pub fn Write_to_device(&self, Device: &Device_type) -> Result_type<()> {
        // Set position to the beginning of the device
        Device.Set_position(&crate::Position_type::Start(0))?;

        // Convert to bytes and write
        let Buffer = self.To_bytes();
        let Bytes_written = Device.Write(&Buffer)?;

        if Bytes_written.As_u64() < Self::SIZE as u64 {
            return Err(Error_type::Input_output);
        }

        Device.Flush()?;
        Ok(())
    }

    /// Check if MBR has a valid signature
    pub fn Is_valid(&self) -> bool {
        self.Boot_signature == Self::SIGNATURE
    }

    /// Get all valid partitions
    pub fn Get_valid_partitions(&self) -> Vec<&MBR_partition_entry> {
        self.Partitions
            .iter()
            .filter(|Partition| Partition.Is_valid())
            .collect()
    }

    /// Get all valid partitions (mutable)
    pub fn Get_valid_partitions_mut(&mut self) -> Vec<&mut MBR_partition_entry> {
        self.Partitions
            .iter_mut()
            .filter(|Partition| Partition.Is_valid())
            .collect()
    }

    /// Get bootable partition (if any)
    pub fn Get_bootable_partition(&self) -> Option<&MBR_partition_entry> {
        self.Partitions
            .iter()
            .find(|Partition| Partition.Is_bootable())
    }

    /// Get bootable partition (mutable, if any)
    pub fn Get_bootable_partition_mut(&mut self) -> Option<&mut MBR_partition_entry> {
        self.Partitions
            .iter_mut()
            .find(|Partition| Partition.Is_bootable())
    }

    /// Set a partition as bootable (clears bootable flag from other partitions)
    pub fn Set_bootable_partition(&mut self, Index: usize) -> Result_type<()> {
        if Index >= Self::MAX_PARTITIONS {
            return Err(Error_type::Invalid_parameter);
        }

        // Clear bootable flag from all partitions
        for Partition in &mut self.Partitions {
            Partition.Set_bootable(false);
        }

        // Set the specified partition as bootable
        self.Partitions[Index].Set_bootable(true);
        Ok(())
    }

    /// Check if this MBR contains a GPT protective partition
    pub fn Has_gpt_protective_partition(&self) -> bool {
        self.Partitions
            .iter()
            .any(|Partition| Partition.Get_partition_type() == Partition_type::Gpt_protective)
    }

    /// Get disk signature as u32
    pub fn Get_disk_signature(&self) -> u32 {
        u32::from_le_bytes(self.Disk_signature)
    }

    /// Set disk signature
    pub fn Set_disk_signature(&mut self, Signature: u32) {
        self.Disk_signature = Signature.to_le_bytes();
    }

    /// Get the first free partition slot
    pub fn Get_free_partition_slot(&self) -> Option<usize> {
        self.Partitions
            .iter()
            .position(|Partition| !Partition.Is_valid())
    }

    /// Add a new partition
    pub fn Add_partition(
        &mut self,
        Partition_type: crate::Partition_type,
        Start_lba: u32,
        Size_sectors: u32,
        Bootable: bool,
    ) -> Result_type<usize> {
        let Slot = self
            .Get_free_partition_slot()
            .ok_or(Error_type::File_system_full)?;

        let New_partition =
            MBR_partition_entry::New_with_params(Bootable, Partition_type, Start_lba, Size_sectors);

        // Check for overlaps with existing partitions
        for Existing in &self.Partitions {
            if Existing.Is_valid() && New_partition.Overlaps_with(Existing) {
                return Err(Error_type::Already_exists);
            }
        }

        self.Partitions[Slot] = New_partition;

        // If this is the only bootable partition or no other bootable partition exists
        if Bootable {
            self.Set_bootable_partition(Slot)?;
        }

        Ok(Slot)
    }

    /// Remove a partition by index
    pub fn Remove_partition(&mut self, Index: usize) -> Result_type<()> {
        if Index >= Self::MAX_PARTITIONS {
            return Err(Error_type::Invalid_parameter);
        }

        self.Partitions[Index].Clear();
        Ok(())
    }

    /// Check for partition overlaps
    pub fn Has_overlapping_partitions(&self) -> bool {
        let Valid_partitions = self.Get_valid_partitions();

        for (I, Partition1) in Valid_partitions.iter().enumerate() {
            for Partition2 in Valid_partitions.iter().skip(I + 1) {
                if Partition1.Overlaps_with(Partition2) {
                    return true;
                }
            }
        }

        false
    }

    /// Get partition count
    pub fn Get_partition_count(&self) -> usize {
        self.Partitions.iter().filter(|P| P.Is_valid()).count()
    }

    /// Convert MBR back to bytes
    pub fn To_bytes(&self) -> [u8; Self::SIZE] {
        let mut Buffer = [0u8; Self::SIZE];

        // Copy bootstrap code
        Buffer[0..440].copy_from_slice(&self.Bootstrap_code);

        // Copy disk signature
        Buffer[440..444].copy_from_slice(&self.Disk_signature);

        // Copy reserved bytes
        Buffer[444..446].copy_from_slice(&self.Reserved);

        // Copy partition entries
        for (I, Partition) in self.Partitions.iter().enumerate() {
            let Offset = 446 + (I * 16);
            Buffer[Offset] = Partition.Bootable;
            Buffer[Offset + 1] = Partition.Start_head;
            Buffer[Offset + 2] = Partition.Start_sector;
            Buffer[Offset + 3] = Partition.Start_cylinder;
            Buffer[Offset + 4] = Partition.Partition_type;
            Buffer[Offset + 5] = Partition.End_head;
            Buffer[Offset + 6] = Partition.End_sector;
            Buffer[Offset + 7] = Partition.End_cylinder;

            let Start_lba_bytes = Partition.Start_lba.to_le_bytes();
            Buffer[Offset + 8..Offset + 12].copy_from_slice(&Start_lba_bytes);

            let Size_bytes = Partition.Size_sectors.to_le_bytes();
            Buffer[Offset + 12..Offset + 16].copy_from_slice(&Size_bytes);
        }

        // Copy boot signature
        Buffer[510..512].copy_from_slice(&self.Boot_signature);

        Buffer
    }
}

impl Default for MBR_type {
    fn default() -> Self {
        Self::New()
    }
}

impl fmt::Display for MBR_type {
    fn fmt(&self, Formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(Formatter, "Master Boot Record:")?;
        writeln!(
            Formatter,
            "  Disk Signature: 0x{:08X}",
            self.Get_disk_signature()
        )?;
        writeln!(
            Formatter,
            "  Boot Signature: 0x{:02X}{:02X}",
            self.Boot_signature[1], self.Boot_signature[0]
        )?;
        writeln!(Formatter, "  Valid: {}", self.Is_valid())?;
        writeln!(
            Formatter,
            "  GPT Protective: {}",
            self.Has_gpt_protective_partition()
        )?;
        writeln!(
            Formatter,
            "  Partition Count: {}",
            self.Get_partition_count()
        )?;
        writeln!(
            Formatter,
            "  Has Overlaps: {}",
            self.Has_overlapping_partitions()
        )?;
        writeln!(Formatter, "  Partitions:")?;

        for (I, Partition) in self.Partitions.iter().enumerate() {
            if Partition.Is_valid() {
                writeln!(Formatter, "    {}: {}", I + 1, Partition)?;
            } else {
                writeln!(Formatter, "    {}: Empty", I + 1)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use crate::MBR_partition_entry;
    use alloc::format;

    fn Create_sample_mbr_bytes() -> [u8; 512] {
        let mut Data = [0u8; 512];

        // Set MBR signature
        Data[510] = 0x55;
        Data[511] = 0xAA;

        // Set disk signature
        Data[440..444].copy_from_slice(&0x12345678u32.to_le_bytes());

        // Create partition 1: FAT32 LBA, bootable, starts at LBA 2048, size 100MB
        let Partition1_offset = 446;
        Data[Partition1_offset] = 0x80; // bootable
        Data[Partition1_offset + 4] = 0x0C; // FAT32 LBA
        Data[Partition1_offset + 8..Partition1_offset + 12].copy_from_slice(&2048u32.to_le_bytes());
        Data[Partition1_offset + 12..Partition1_offset + 16]
            .copy_from_slice(&204800u32.to_le_bytes());

        // Create partition 2: Linux, starts after partition 1
        let Partition2_offset = 446 + 16;
        Data[Partition2_offset + 4] = 0x83; // Linux
        Data[Partition2_offset + 8..Partition2_offset + 12]
            .copy_from_slice(&206848u32.to_le_bytes());
        Data[Partition2_offset + 12..Partition2_offset + 16]
            .copy_from_slice(&102400u32.to_le_bytes());

        Data
    }

    #[test]
    fn Test_mbr_new() {
        let Mbr = super::MBR_type::New();
        assert!(Mbr.Is_valid());
        assert_eq!(Mbr.Get_disk_signature(), 0);
        assert_eq!(Mbr.Get_partition_count(), 0);
        assert!(!Mbr.Has_gpt_protective_partition());
        assert!(!Mbr.Has_overlapping_partitions());
    }

    #[test]
    fn Test_mbr_new_with_signature() {
        let Signature = 0xDEADBEEF;
        let Mbr = MBR_type::New_with_signature(Signature);
        assert!(Mbr.Is_valid());
        assert_eq!(Mbr.Get_disk_signature(), Signature);
    }

    #[test]
    fn Test_mbr_from_bytes() {
        let Data = Create_sample_mbr_bytes();
        let Mbr = MBR_type::From_bytes(&Data).expect("Should parse MBR successfully");

        assert!(Mbr.Is_valid());
        assert_eq!(Mbr.Get_disk_signature(), 0x12345678);
        assert_eq!(Mbr.Get_partition_count(), 2);

        let Partitions = Mbr.Get_valid_partitions();
        assert_eq!(Partitions.len(), 2);

        // Check first partition
        let P1 = &Partitions[0];
        assert!(P1.Is_bootable());
        assert_eq!(P1.Get_partition_type(), super::Partition_type::Fat32_lba);
        assert_eq!(P1.Get_start_lba(), 2048);
        assert_eq!(P1.Get_size_sectors(), 204800);

        // Check second partition
        let P2 = &Partitions[1];
        assert!(!P2.Is_bootable());
        assert_eq!(P2.Get_partition_type(), super::Partition_type::Linux);
        assert_eq!(P2.Get_start_lba(), 206848);
        assert_eq!(P2.Get_size_sectors(), 102400);
    }

    #[test]
    fn Test_mbr_from_bytes_invalid_signature() {
        let mut Data = Create_sample_mbr_bytes();
        Data[510] = 0x00; // Invalid signature
        Data[511] = 0x00;

        let Result = MBR_type::From_bytes(&Data);
        assert!(Result.is_err());
        assert_eq!(Result.unwrap_err(), crate::Error_type::Corrupted);
    }

    #[test]
    fn Test_mbr_from_bytes_too_small() {
        let Data = [0u8; 256]; // Too small
        let Result = MBR_type::From_bytes(&Data);
        assert!(Result.is_err());
        assert_eq!(Result.unwrap_err(), crate::Error_type::Invalid_parameter);
    }

    #[test]
    fn Test_mbr_to_bytes_round_trip() {
        let Original_data = Create_sample_mbr_bytes();
        let Mbr = MBR_type::From_bytes(&Original_data).unwrap();
        let Serialized_data = Mbr.To_bytes();

        assert_eq!(Original_data.len(), Serialized_data.len());
        assert_eq!(Original_data, Serialized_data);
    }

    #[test]
    fn Test_mbr_disk_signature() {
        let mut Mbr = super::MBR_type::New();
        assert_eq!(Mbr.Get_disk_signature(), 0);

        Mbr.Set_disk_signature(0xCAFEBABE);
        assert_eq!(Mbr.Get_disk_signature(), 0xCAFEBABE);
    }

    #[test]
    fn Test_mbr_get_bootable_partition() {
        let Data = Create_sample_mbr_bytes();
        let Mbr = MBR_type::From_bytes(&Data).unwrap();

        let Bootable = Mbr.Get_bootable_partition();
        assert!(Bootable.is_some());
        assert_eq!(
            Bootable.unwrap().Get_partition_type(),
            super::Partition_type::Fat32_lba
        );
    }

    #[test]
    fn Test_mbr_set_bootable_partition() {
        let mut Mbr = super::MBR_type::New();

        // Add two partitions
        Mbr.Add_partition(super::Partition_type::Fat32, 2048, 100000, false)
            .unwrap();
        Mbr.Add_partition(super::Partition_type::Linux, 102048, 50000, true)
            .unwrap();

        // Second partition should be bootable
        let Bootable = Mbr.Get_bootable_partition().unwrap();
        assert_eq!(Bootable.Get_partition_type(), super::Partition_type::Linux);

        // Set first partition as bootable
        Mbr.Set_bootable_partition(0).unwrap();
        let Bootable = Mbr.Get_bootable_partition().unwrap();
        assert_eq!(Bootable.Get_partition_type(), super::Partition_type::Fat32);
    }

    #[test]
    fn Test_mbr_add_partition() {
        let mut Mbr = super::MBR_type::New();

        let Index = Mbr
            .Add_partition(super::Partition_type::Fat32_lba, 2048, 204800, true)
            .unwrap();

        assert_eq!(Index, 0);
        assert_eq!(Mbr.Get_partition_count(), 1);

        let Partition = &Mbr.Partitions[Index];
        assert!(Partition.Is_valid());
        assert!(Partition.Is_bootable());
        assert_eq!(
            Partition.Get_partition_type(),
            super::Partition_type::Fat32_lba
        );
    }

    #[test]
    fn Test_mbr_add_overlapping_partition() {
        let mut Mbr = super::MBR_type::New();

        // Add first partition
        Mbr.Add_partition(super::Partition_type::Fat32, 1000, 2000, false)
            .unwrap();

        // Try to add overlapping partition
        let Result = Mbr.Add_partition(super::Partition_type::Linux, 1500, 1000, false);
        assert!(Result.is_err());
        assert_eq!(Result.unwrap_err(), crate::Error_type::Already_exists);
    }

    #[test]
    fn Test_mbr_add_too_many_partitions() {
        let mut Mbr = super::MBR_type::New();

        // Fill all 4 partition slots
        for I in 0..4 {
            let Start = (I as u32) * 10000 + 1000;
            Mbr.Add_partition(super::Partition_type::Linux, Start, 5000, false)
                .unwrap();
        }

        // Try to add fifth partition
        let Result = Mbr.Add_partition(super::Partition_type::Fat32, 50000, 1000, false);
        assert!(Result.is_err());
        assert_eq!(Result.unwrap_err(), crate::Error_type::File_system_full);
    }

    #[test]
    fn Test_mbr_remove_partition() {
        let mut Mbr = super::MBR_type::New();

        Mbr.Add_partition(super::Partition_type::Fat32, 2048, 100000, false)
            .unwrap();
        assert_eq!(Mbr.Get_partition_count(), 1);

        Mbr.Remove_partition(0).unwrap();
        assert_eq!(Mbr.Get_partition_count(), 0);
    }

    #[test]
    fn Test_mbr_remove_invalid_partition() {
        let mut Mbr = super::MBR_type::New();
        let Result = Mbr.Remove_partition(5); // Invalid index
        assert!(Result.is_err());
        assert_eq!(Result.unwrap_err(), crate::Error_type::Invalid_parameter);
    }

    #[test]
    fn Test_mbr_free_partition_slot() {
        let mut Mbr = super::MBR_type::New();

        // Should have first slot free
        assert_eq!(Mbr.Get_free_partition_slot(), Some(0));

        // Fill first two slots
        Mbr.Add_partition(super::Partition_type::Fat32, 1000, 1000, false)
            .unwrap();
        Mbr.Add_partition(super::Partition_type::Linux, 3000, 1000, false)
            .unwrap();

        // Should have third slot free
        assert_eq!(Mbr.Get_free_partition_slot(), Some(2));

        // Fill remaining slots
        Mbr.Add_partition(super::Partition_type::Linux_swap, 5000, 1000, false)
            .unwrap();
        Mbr.Add_partition(super::Partition_type::Ntfs_exfat, 7000, 1000, false)
            .unwrap();

        // Should have no free slots
        assert_eq!(Mbr.Get_free_partition_slot(), None);
    }

    #[test]
    fn Test_mbr_has_gpt_protective() {
        let mut Mbr = super::MBR_type::New();
        assert!(!Mbr.Has_gpt_protective_partition());

        Mbr.Add_partition(super::Partition_type::Gpt_protective, 1, 0xFFFFFFFF, false)
            .unwrap();
        assert!(Mbr.Has_gpt_protective_partition());
    }

    #[test]
    fn Test_mbr_overlapping_partitions_detection() {
        let mut Mbr = super::MBR_type::New();

        // Add non-overlapping partitions
        Mbr.Add_partition(super::Partition_type::Fat32, 1000, 1000, false)
            .unwrap();
        Mbr.Add_partition(super::Partition_type::Linux, 3000, 1000, false)
            .unwrap();
        assert!(!Mbr.Has_overlapping_partitions());

        // Manually create overlapping partitions (bypassing validation)
        Mbr.Partitions[2] = MBR_partition_entry::New_with_params(
            false,
            super::Partition_type::Linux_swap,
            1500,
            1000,
        );
        assert!(Mbr.Has_overlapping_partitions());
    }

    #[test]
    fn Test_mbr_default() {
        let Mbr = super::MBR_type::default();
        assert!(Mbr.Is_valid());
        assert_eq!(Mbr.Get_partition_count(), 0);
    }

    #[test]
    fn Test_mbr_display() {
        let Data = Create_sample_mbr_bytes();
        let Mbr = MBR_type::From_bytes(&Data).unwrap();

        let Display_string = format!("{Mbr}");
        assert!(Display_string.contains("Master Boot Record"));
        assert!(Display_string.contains("Disk Signature: 0x12345678"));
        assert!(Display_string.contains("Valid: true"));
        assert!(Display_string.contains("Partition Count: 2"));
        assert!(Display_string.contains("FAT32 LBA"));
        assert!(Display_string.contains("Linux"));
    }

    #[test]
    fn Test_mbr_constants() {
        assert_eq!(super::MBR_type::SIZE, 512);
        assert_eq!(super::MBR_type::MAX_PARTITIONS, 4);
        assert_eq!(super::MBR_type::SIGNATURE, [0x55, 0xAA]);
    }
}
