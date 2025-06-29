use alloc::vec::Vec;

use super::MBR_type;
use crate::{Device_type, Error_type, MBR_partition_entry, Partition_device_type, Result_type};

/// Create a partition device from an MBR partition entry
pub fn Create_partition_device(
    Base_device: Device_type,
    Partition: &MBR_partition_entry,
) -> Result_type<Partition_device_type> {
    if !Partition.Is_valid() {
        return Err(Error_type::Invalid_parameter);
    }

    Ok(Partition_device_type::New_from_lba(
        Base_device,
        Partition.Get_start_lba(),
        Partition.Get_size_sectors(),
    ))
}

/// Scan a device for MBR and return partition information
pub fn Scan_mbr_partitions(Device: &Device_type) -> Result_type<Vec<(usize, MBR_partition_entry)>> {
    let Mbr = MBR_type::Read_from_device(Device)?;

    let mut Partitions = Vec::new();
    for (I, Partition) in Mbr.Partitions.iter().enumerate() {
        if Partition.Is_valid() {
            Partitions.push((I, *Partition));
        }
    }

    Ok(Partitions)
}

/// Validate MBR structure and partitions
pub fn Validate_mbr(Mbr: &crate::MBR_type) -> Result_type<()> {
    // Check MBR signature
    if !Mbr.Is_valid() {
        return Err(Error_type::Corrupted);
    }

    // Check for overlapping partitions
    if Mbr.Has_overlapping_partitions() {
        return Err(Error_type::Corrupted);
    }

    // Check that only one partition is bootable
    let Bootable_count = Mbr.Partitions.iter().filter(|P| P.Is_bootable()).count();

    if Bootable_count > 1 {
        return Err(Error_type::Corrupted);
    }

    Ok(())
}

/// Create all partition devices from an MBR
pub fn Create_all_partition_devices(
    Base_device: Device_type,
    Mbr: &super::MBR_type,
) -> Result_type<Vec<Partition_device_type>> {
    let mut Devices = Vec::new();

    for Partition in &Mbr.Partitions {
        if Partition.Is_valid() {
            let Device = Create_partition_device(Base_device.clone(), Partition)?;
            Devices.push(Device);
        }
    }

    Ok(Devices)
}

/// Find partitions of a specific type
pub fn Find_partitions_by_type(
    Mbr: &super::MBR_type,
    Partition_type: crate::Partition_type,
) -> Vec<(usize, &MBR_partition_entry)> {
    Mbr.Partitions
        .iter()
        .enumerate()
        .filter(|(_, Partition)| {
            Partition.Is_valid() && Partition.Get_partition_type() == Partition_type
        })
        .collect()
}

/// Get partition statistics
#[derive(Debug, Clone)]
pub struct Partition_statistics {
    pub Total_partitions: usize,
    pub Bootable_partitions: usize,
    pub Fat_partitions: usize,
    pub Linux_partitions: usize,
    pub Hidden_partitions: usize,
    pub Extended_partitions: usize,
    pub Unknown_partitions: usize,
    pub Total_used_sectors: u64,
    pub Largest_partition_sectors: u32,
    pub Smallest_partition_sectors: u32,
}

impl Partition_statistics {
    pub fn From_mbr(Mbr: &super::MBR_type) -> Self {
        let Valid_partitions: Vec<_> = Mbr.Get_valid_partitions();

        let Total_partitions = Valid_partitions.len();
        let Bootable_partitions = Valid_partitions.iter().filter(|P| P.Is_bootable()).count();

        let Fat_partitions = Valid_partitions
            .iter()
            .filter(|P| P.Get_partition_type().Is_fat())
            .count();

        let Linux_partitions = Valid_partitions
            .iter()
            .filter(|P| P.Get_partition_type().Is_linux())
            .count();

        let Hidden_partitions = Valid_partitions
            .iter()
            .filter(|P| P.Get_partition_type().Is_hidden())
            .count();

        let Extended_partitions = Valid_partitions
            .iter()
            .filter(|P| P.Get_partition_type().Is_extended())
            .count();

        let Unknown_partitions = Valid_partitions
            .iter()
            .filter(|P| matches!(P.Get_partition_type(), crate::Partition_type::Unknown(_)))
            .count();

        let Total_used_sectors = Valid_partitions
            .iter()
            .map(|P| P.Get_size_sectors() as u64)
            .sum();

        let Largest_partition_sectors = Valid_partitions
            .iter()
            .map(|P| P.Get_size_sectors())
            .max()
            .unwrap_or(0);

        let Smallest_partition_sectors = Valid_partitions
            .iter()
            .map(|P| P.Get_size_sectors())
            .min()
            .unwrap_or(0);

        Self {
            Total_partitions,
            Bootable_partitions,
            Fat_partitions,
            Linux_partitions,
            Hidden_partitions,
            Extended_partitions,
            Unknown_partitions,
            Total_used_sectors,
            Largest_partition_sectors,
            Smallest_partition_sectors,
        }
    }
}

/// Check if a device contains a valid MBR
pub fn Has_valid_mbr(Device: &Device_type) -> bool {
    match MBR_type::Read_from_device(Device) {
        Ok(Mbr) => Mbr.Is_valid(),
        Err(_) => false,
    }
}

/// Check if a device uses GPT (has GPT protective partition)
pub fn Is_gpt_disk(Device: &Device_type) -> bool {
    match MBR_type::Read_from_device(Device) {
        Ok(Mbr) => Mbr.Has_gpt_protective_partition(),
        Err(_) => false,
    }
}

/// Create a basic MBR with a single partition
pub fn Create_basic_mbr(
    Disk_signature: u32,
    Partition_type: crate::Partition_type,
    Total_sectors: u32,
) -> Result_type<super::MBR_type> {
    let mut Mbr = MBR_type::New_with_signature(Disk_signature);

    // Leave some space at the beginning (typically 2048 sectors for alignment)
    let Start_lba = 2048;
    let Partition_sectors = Total_sectors.saturating_sub(Start_lba);

    if Partition_sectors == 0 {
        return Err(Error_type::Invalid_parameter);
    }

    Mbr.Add_partition(Partition_type, Start_lba, Partition_sectors, true)?;

    Ok(Mbr)
}

/// Clone an MBR to another device
pub fn Clone_mbr(Source_device: &Device_type, Target_device: &Device_type) -> Result_type<()> {
    let Mbr = MBR_type::Read_from_device(Source_device)?;
    Validate_mbr(&Mbr)?;
    Mbr.Write_to_device(Target_device)?;
    Ok(())
}

/// Backup MBR to a buffer
pub fn Backup_mbr(Device: &Device_type) -> Result_type<[u8; 512]> {
    let Mbr = MBR_type::Read_from_device(Device)?;
    Ok(Mbr.To_bytes())
}

/// Restore MBR from a buffer
pub fn Restore_mbr(Device: &Device_type, Backup: &[u8; 512]) -> Result_type<()> {
    let Mbr = MBR_type::From_bytes(Backup)?;
    Validate_mbr(&Mbr)?;
    Mbr.Write_to_device(Device)?;
    Ok(())
}

#[cfg(test)]
mod Tests {
    use super::*;
    use crate::{Device_type, Error_type, Memory_device_type, Partition_type};
    use alloc::{format, sync::Arc, vec};

    /// Create a test device with MBR data
    fn Create_test_device_with_mbr() -> Device_type {
        let mut Data = vec![0u8; 4096];

        // Create a simple MBR at the beginning
        let Mbr = create_test_mbr();
        let Mbr_bytes = Mbr.To_bytes();
        Data[0..512].copy_from_slice(&Mbr_bytes);

        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        crate::Create_device!(Memory_device)
    }

    /// Create a test device without valid MBR
    fn Create_test_device_no_mbr() -> Device_type {
        let Memory_device = Memory_device_type::<512>::New(4096);
        crate::Create_device!(Memory_device)
    }

    /// Create a test MBR for testing
    fn create_test_mbr() -> MBR_type {
        let mut Mbr = MBR_type::New_with_signature(0x12345678);

        // Add a few test partitions
        let _ = Mbr.Add_partition(Partition_type::Fat32_lba, 2048, 1024, true);
        let _ = Mbr.Add_partition(Partition_type::Linux, 4096, 2048, false);
        let _ = Mbr.Add_partition(Partition_type::Hidden_fat16, 8192, 512, false);

        Mbr
    }

    #[test]
    fn Test_create_partition_device() {
        let Base_device = Create_test_device_with_mbr();
        let Mbr = MBR_type::Read_from_device(&Base_device).unwrap();
        let Partition = &Mbr.Partitions[0];

        let Device_result = Create_partition_device(Base_device, Partition);
        assert!(Device_result.is_ok());

        let Device = Device_result.unwrap();
        assert_eq!(Device.Get_start_lba(), Partition.Get_start_lba());
        assert_eq!(Device.Get_sector_count(), Partition.Get_size_sectors());
        assert!(Device.Is_valid());
    }

    #[test]
    fn Test_create_partition_device_invalid() {
        let Base_device = Create_test_device_with_mbr();
        let Invalid_partition = MBR_partition_entry::New();

        let Device_result = Create_partition_device(Base_device, &Invalid_partition);
        assert!(Device_result.is_err());
        assert_eq!(Device_result.unwrap_err(), Error_type::Invalid_parameter);
    }

    #[test]
    fn Test_scan_mbr_partitions() {
        let Device = Create_test_device_with_mbr();

        let Scan_result = Scan_mbr_partitions(&Device);
        assert!(Scan_result.is_ok());

        let Partitions = Scan_result.unwrap();
        assert_eq!(Partitions.len(), 3); // Should find 3 valid partitions

        // Check that indices are correct
        assert_eq!(Partitions[0].0, 0);
        assert_eq!(Partitions[1].0, 1);
        assert_eq!(Partitions[2].0, 2);

        // Check partition types
        assert_eq!(
            Partitions[0].1.Get_partition_type(),
            Partition_type::Fat32_lba
        );
        assert_eq!(Partitions[1].1.Get_partition_type(), Partition_type::Linux);
        assert_eq!(
            Partitions[2].1.Get_partition_type(),
            Partition_type::Hidden_fat16
        );
    }

    #[test]
    fn Test_scan_mbr_partitions_no_mbr() {
        let Device = Create_test_device_no_mbr();

        let Scan_result = Scan_mbr_partitions(&Device);
        assert!(Scan_result.is_err());
    }

    #[test]
    fn Test_validate_mbr_valid() {
        let Mbr = create_test_mbr();
        let Validation_result = Validate_mbr(&Mbr);
        assert!(Validation_result.is_ok());
    }

    #[test]
    fn Test_validate_mbr_invalid_signature() {
        let mut Mbr = super::MBR_type::New();
        // Manually set invalid signature - should be invalid
        Mbr.Boot_signature = [0x00, 0x00];

        let Validation_result = Validate_mbr(&Mbr);
        assert!(Validation_result.is_err());
        assert_eq!(Validation_result.unwrap_err(), Error_type::Corrupted);
    }

    #[test]
    fn Test_validate_mbr_multiple_bootable() {
        let mut Mbr = MBR_type::New_with_signature(0x12345678);

        // Manually create multiple bootable partitions (bypassing Add_partition validation)
        Mbr.Partitions[0] =
            MBR_partition_entry::New_with_params(true, Partition_type::Fat32_lba, 2048, 1024);
        Mbr.Partitions[1] =
            MBR_partition_entry::New_with_params(true, Partition_type::Linux, 4096, 2048);

        let Validation_result = Validate_mbr(&Mbr);
        assert!(Validation_result.is_err());
        assert_eq!(Validation_result.unwrap_err(), Error_type::Corrupted);
    }

    #[test]
    fn Test_validate_mbr_overlapping_partitions() {
        let mut Mbr = MBR_type::New_with_signature(0x12345678);

        // Manually create overlapping partitions (bypassing Add_partition validation)
        Mbr.Partitions[0] =
            MBR_partition_entry::New_with_params(false, Partition_type::Fat32_lba, 2048, 2048);
        Mbr.Partitions[1] =
            MBR_partition_entry::New_with_params(false, Partition_type::Linux, 3000, 2048);

        let Validation_result = Validate_mbr(&Mbr);
        assert!(Validation_result.is_err());
        assert_eq!(Validation_result.unwrap_err(), Error_type::Corrupted);
    }

    #[test]
    fn Test_create_all_partition_devices() {
        let Base_device = Create_test_device_with_mbr();
        let Mbr = create_test_mbr();

        let Devices_result = Create_all_partition_devices(Base_device, &Mbr);
        assert!(Devices_result.is_ok());

        let Devices = Devices_result.unwrap();
        assert_eq!(Devices.len(), 3); // Should create 3 devices

        // Check that all devices are valid
        for Device in &Devices {
            assert!(Device.Is_valid());
        }

        // Check first device properties
        assert_eq!(Devices[0].Get_start_lba(), 2048);
        assert_eq!(Devices[0].Get_sector_count(), 1024);
    }

    #[test]
    fn Test_find_partitions_by_type() {
        let Mbr = create_test_mbr();

        // Find FAT32 partitions
        let Fat_partitions = Find_partitions_by_type(&Mbr, Partition_type::Fat32_lba);
        assert_eq!(Fat_partitions.len(), 1);
        assert_eq!(Fat_partitions[0].0, 0); // Index 0

        // Find Linux partitions
        let Linux_partitions = Find_partitions_by_type(&Mbr, Partition_type::Linux);
        assert_eq!(Linux_partitions.len(), 1);
        assert_eq!(Linux_partitions[0].0, 1); // Index 1

        // Find non-existent type
        let Ntfs_partitions = Find_partitions_by_type(&Mbr, Partition_type::Ntfs_exfat);
        assert_eq!(Ntfs_partitions.len(), 0);
    }

    #[test]
    fn Test_partition_statistics() {
        let Mbr = create_test_mbr();
        let Stats = Partition_statistics::From_mbr(&Mbr);

        assert_eq!(Stats.Total_partitions, 3);
        assert_eq!(Stats.Bootable_partitions, 1);
        assert_eq!(Stats.Fat_partitions, 2); // Fat32_lba + Hidden_fat16
        assert_eq!(Stats.Linux_partitions, 1);
        assert_eq!(Stats.Hidden_partitions, 1); // Hidden_fat16
        assert_eq!(Stats.Extended_partitions, 0);
        assert_eq!(Stats.Unknown_partitions, 0);
        assert_eq!(Stats.Total_used_sectors, 1024 + 2048 + 512); // Sum of all partition sizes
        assert_eq!(Stats.Largest_partition_sectors, 2048);
        assert_eq!(Stats.Smallest_partition_sectors, 512);
    }

    #[test]
    fn Test_partition_statistics_empty_mbr() {
        let Mbr = MBR_type::New_with_signature(0x12345678);
        let Stats = Partition_statistics::From_mbr(&Mbr);

        assert_eq!(Stats.Total_partitions, 0);
        assert_eq!(Stats.Bootable_partitions, 0);
        assert_eq!(Stats.Fat_partitions, 0);
        assert_eq!(Stats.Linux_partitions, 0);
        assert_eq!(Stats.Hidden_partitions, 0);
        assert_eq!(Stats.Extended_partitions, 0);
        assert_eq!(Stats.Unknown_partitions, 0);
        assert_eq!(Stats.Total_used_sectors, 0);
        assert_eq!(Stats.Largest_partition_sectors, 0);
        assert_eq!(Stats.Smallest_partition_sectors, 0);
    }

    #[test]
    fn Test_has_valid_mbr() {
        let Valid_device = Create_test_device_with_mbr();
        assert!(Has_valid_mbr(&Valid_device));

        let Invalid_device = Create_test_device_no_mbr();
        assert!(!Has_valid_mbr(&Invalid_device));
    }

    #[test]
    fn Test_is_gpt_disk() {
        // Create MBR with GPT protective partition
        let mut Mbr = MBR_type::New_with_signature(0x12345678);
        let _ = Mbr.Add_partition(Partition_type::Gpt_protective, 1, 0xFFFFFFFF, false);

        let mut Data = vec![0u8; 4096];
        let Mbr_bytes = Mbr.To_bytes();
        Data[0..512].copy_from_slice(&Mbr_bytes);
        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        let Gpt_device = crate::Create_device!(Memory_device);

        assert!(Is_gpt_disk(&Gpt_device));

        // Regular MBR should not be detected as GPT
        let Regular_device = Create_test_device_with_mbr();
        assert!(!Is_gpt_disk(&Regular_device));
    }

    #[test]
    fn Test_create_basic_mbr() {
        let Mbr_result = Create_basic_mbr(0xABCDEF00, Partition_type::Fat32_lba, 100000);
        assert!(Mbr_result.is_ok());

        let Mbr = Mbr_result.unwrap();
        assert!(Mbr.Is_valid());
        assert_eq!(Mbr.Get_disk_signature(), 0xABCDEF00);

        let Valid_partitions = Mbr.Get_valid_partitions();
        assert_eq!(Valid_partitions.len(), 1);

        let Partition = &Valid_partitions[0];
        assert_eq!(Partition.Get_partition_type(), Partition_type::Fat32_lba);
        assert_eq!(Partition.Get_start_lba(), 2048);
        assert_eq!(Partition.Get_size_sectors(), 100000 - 2048);
        assert!(Partition.Is_bootable());
    }

    #[test]
    fn Test_create_basic_mbr_too_small() {
        let Mbr_result = Create_basic_mbr(0x12345678, Partition_type::Fat32_lba, 1000);
        assert!(Mbr_result.is_err());
        assert_eq!(Mbr_result.unwrap_err(), Error_type::Invalid_parameter);
    }

    #[test]
    fn Test_clone_mbr() {
        let Source_device = Create_test_device_with_mbr();
        let Target_data = vec![0u8; 4096];
        let Memory_device = Memory_device_type::<512>::From_vec(Target_data);
        let Target_device = crate::Create_device!(Memory_device);

        let Clone_result = Clone_mbr(&Source_device, &Target_device);
        assert!(Clone_result.is_ok());

        // Verify that target device now has the same MBR
        let Source_mbr = MBR_type::Read_from_device(&Source_device).unwrap();
        let Target_mbr = MBR_type::Read_from_device(&Target_device).unwrap();

        assert_eq!(Source_mbr.To_bytes(), Target_mbr.To_bytes());
    }

    #[test]
    fn Test_backup_and_restore_mbr() {
        let Device = Create_test_device_with_mbr();

        // Backup MBR
        let Backup_result = Backup_mbr(&Device);
        assert!(Backup_result.is_ok());
        let Backup = Backup_result.unwrap();

        // Create a new device with zeros
        let Target_data = vec![0u8; 4096];
        let Memory_device = Memory_device_type::<512>::From_vec(Target_data);
        let Target_device = crate::Create_device!(Memory_device);

        // Restore MBR
        let Restore_result = Restore_mbr(&Target_device, &Backup);
        assert!(Restore_result.is_ok());

        // Verify restoration
        let Original_mbr = MBR_type::Read_from_device(&Device).unwrap();
        let Restored_mbr = MBR_type::Read_from_device(&Target_device).unwrap();

        assert_eq!(Original_mbr.To_bytes(), Restored_mbr.To_bytes());
    }

    #[test]
    fn Test_backup_mbr_invalid_device() {
        let Invalid_device = Create_test_device_no_mbr();

        let Backup_result = Backup_mbr(&Invalid_device);
        assert!(Backup_result.is_err());
    }

    #[test]
    fn Test_restore_mbr_invalid_backup() {
        let Device = Create_test_device_no_mbr();
        let Invalid_backup = [0u8; 512]; // No valid signature

        let Restore_result = Restore_mbr(&Device, &Invalid_backup);
        assert!(Restore_result.is_err());
    }

    #[test]
    fn Test_utilities_with_complex_mbr() {
        // Create a more complex MBR for comprehensive testing
        let mut Mbr = MBR_type::New_with_signature(0xDEADBEEF);

        let _ = Mbr.Add_partition(Partition_type::Fat16, 63, 1000, true);
        let _ = Mbr.Add_partition(Partition_type::Extended_lba, 2048, 10000, false);
        let _ = Mbr.Add_partition(Partition_type::Linux_swap, 15000, 2000, false);
        let _ = Mbr.Add_partition(Partition_type::Unknown(0x42), 20000, 5000, false);

        // Test statistics
        let Stats = Partition_statistics::From_mbr(&Mbr);
        assert_eq!(Stats.Total_partitions, 4);
        assert_eq!(Stats.Bootable_partitions, 1);
        assert_eq!(Stats.Fat_partitions, 1);
        assert_eq!(Stats.Linux_partitions, 1); // Linux_swap counts as Linux
        assert_eq!(Stats.Extended_partitions, 1);
        assert_eq!(Stats.Unknown_partitions, 1);

        // Test finding by type
        let Extended = Find_partitions_by_type(&Mbr, Partition_type::Extended_lba);
        assert_eq!(Extended.len(), 1);
        assert_eq!(Extended[0].0, 1);

        let Unknown = Find_partitions_by_type(&Mbr, Partition_type::Unknown(0x42));
        assert_eq!(Unknown.len(), 1);
        assert_eq!(Unknown[0].0, 3);
    }

    #[test]
    fn Test_edge_cases() {
        // Test with MBR containing only empty partitions
        let Empty_mbr = MBR_type::New_with_signature(0x12345678);

        let Stats = Partition_statistics::From_mbr(&Empty_mbr);
        assert_eq!(Stats.Total_partitions, 0);

        let Partitions = Find_partitions_by_type(&Empty_mbr, Partition_type::Fat32);
        assert_eq!(Partitions.len(), 0);

        // Test scan on empty device
        let mut Empty_data = vec![0u8; 4096];
        let Empty_mbr_bytes = Empty_mbr.To_bytes();
        Empty_data[0..512].copy_from_slice(&Empty_mbr_bytes);
        let Memory_device = Memory_device_type::<512>::From_vec(Empty_data);
        let Empty_device = crate::Create_device!(Memory_device);

        let Scan_result = Scan_mbr_partitions(&Empty_device);
        assert!(Scan_result.is_ok());
        assert_eq!(Scan_result.unwrap().len(), 0);
    }
}
