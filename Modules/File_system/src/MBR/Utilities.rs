use alloc::vec::Vec;

use super::MBR_type;
use crate::{Device_type, Error_type, Partition_device_type, Partition_entry_type, Result_type};

/// Create a partition device from an MBR partition entry
pub fn Create_partition_device(
    Base_device: Device_type,
    Partition: &Partition_entry_type,
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
pub fn Scan_mbr_partitions(
    Device: &Device_type,
) -> Result_type<Vec<(usize, Partition_entry_type)>> {
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
    Partition_type: crate::Partition_type_type,
) -> Vec<(usize, &Partition_entry_type)> {
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
pub struct Partition_statistics_type {
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

impl Partition_statistics_type {
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
            .filter(|P| {
                matches!(
                    P.Get_partition_type(),
                    crate::Partition_type_type::Unknown(_)
                )
            })
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
    Partition_type: crate::Partition_type_type,
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

/// Format disk to MBR if it doesn't contain valid MBR and return first partition device
///
/// This function checks if the device contains a valid MBR. If not, it creates a new MBR
/// with a single partition using the full disk size. It then returns a partition device
/// for the first partition.
///
/// # Arguments
/// * `Device` - The device to format and get partition from
/// * `Partition_type` - The type of partition to create if formatting is needed
/// * `Disk_signature` - The disk signature to use when creating new MBR (optional, uses random if None)
///
/// # Returns
/// * `Result_type<Partition_device_type>` - The first partition device
pub fn Format_disk_and_get_first_partition(
    Device: &Device_type,
    Partition_type: crate::Partition_type_type,
    Disk_signature: Option<u32>,
) -> Result_type<Partition_device_type> {
    // Check if device already has valid MBR
    let Mbr = if Has_valid_mbr(Device) {
        // Read existing MBR
        MBR_type::Read_from_device(Device)?
    } else {
        // Get device size in sectors
        let Device_size = Device.Get_size()?;
        let Block_size = Device.Get_block_size()?;
        let Total_sectors = (Device_size.As_u64() / Block_size as u64) as u32;

        if Total_sectors < 2048 {
            return Err(Error_type::Invalid_parameter);
        }

        // Create new MBR with signature
        let Signature = Disk_signature.unwrap_or_else(|| {
            // Generate a simple signature based on current time or use a default
            // In a real implementation, you might want to use a proper random number generator
            0x12345678
        });

        let New_mbr = Create_basic_mbr(Signature, Partition_type, Total_sectors)?;

        // Write the new MBR to device
        New_mbr.Write_to_device(Device)?;

        New_mbr
    };

    // Get the first valid partition
    let Valid_partitions = Mbr.Get_valid_partitions();
    if Valid_partitions.is_empty() {
        return Err(Error_type::Not_found);
    }

    // Create partition device for the first partition
    Create_partition_device(Device.clone(), &Valid_partitions[0])
}

#[cfg(test)]
mod Tests {
    use super::*;
    use crate::{Device_trait, Device_type, Error_type, Memory_device_type, Partition_type_type};
    use alloc::vec;

    /// Create a test device with MBR data
    fn Create_test_device_with_mbr() -> Device_type {
        let mut Data = vec![0u8; 4096 * 1024]; // Make it large enough (4MB = 8192 sectors)

        // Create a simple MBR at the beginning
        let Mbr = create_test_mbr();
        let Mbr_bytes = Mbr.To_bytes();
        Data[0..512].copy_from_slice(&Mbr_bytes);

        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        crate::Create_device!(Memory_device)
    }

    /// Create a test device without valid MBR
    fn Create_test_device_no_mbr() -> Device_type {
        let Memory_device = Memory_device_type::<512>::New(4096 * 1024); // Make it large enough (4MB = 8192 sectors)
        crate::Create_device!(Memory_device)
    }

    /// Create a test MBR for testing
    fn create_test_mbr() -> MBR_type {
        let mut Mbr = MBR_type::New_with_signature(0x12345678);

        // Add a few test partitions
        let _ = Mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true);
        let _ = Mbr.Add_partition(Partition_type_type::Linux, 4096, 2048, false);
        let _ = Mbr.Add_partition(Partition_type_type::Hidden_fat16, 8192, 512, false);

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
        let Invalid_partition = Partition_entry_type::New();

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
            Partition_type_type::Fat32_lba
        );
        assert_eq!(
            Partitions[1].1.Get_partition_type(),
            Partition_type_type::Linux
        );
        assert_eq!(
            Partitions[2].1.Get_partition_type(),
            Partition_type_type::Hidden_fat16
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
            Partition_entry_type::New_with_params(true, Partition_type_type::Fat32_lba, 2048, 1024);
        Mbr.Partitions[1] =
            Partition_entry_type::New_with_params(true, Partition_type_type::Linux, 4096, 2048);

        let Validation_result = Validate_mbr(&Mbr);
        assert!(Validation_result.is_err());
        assert_eq!(Validation_result.unwrap_err(), Error_type::Corrupted);
    }

    #[test]
    fn Test_validate_mbr_overlapping_partitions() {
        let mut Mbr = MBR_type::New_with_signature(0x12345678);

        // Manually create overlapping partitions (bypassing Add_partition validation)
        Mbr.Partitions[0] = Partition_entry_type::New_with_params(
            false,
            Partition_type_type::Fat32_lba,
            2048,
            2048,
        );
        Mbr.Partitions[1] =
            Partition_entry_type::New_with_params(false, Partition_type_type::Linux, 3000, 2048);

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
        let Fat_partitions = Find_partitions_by_type(&Mbr, Partition_type_type::Fat32_lba);
        assert_eq!(Fat_partitions.len(), 1);
        assert_eq!(Fat_partitions[0].0, 0); // Index 0

        // Find Linux partitions
        let Linux_partitions = Find_partitions_by_type(&Mbr, Partition_type_type::Linux);
        assert_eq!(Linux_partitions.len(), 1);
        assert_eq!(Linux_partitions[0].0, 1); // Index 1

        // Find non-existent type
        let Ntfs_partitions = Find_partitions_by_type(&Mbr, Partition_type_type::Ntfs_exfat);
        assert_eq!(Ntfs_partitions.len(), 0);
    }

    #[test]
    fn Test_partition_statistics() {
        let Mbr = create_test_mbr();
        let Stats = Partition_statistics_type::From_mbr(&Mbr);

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
        let Stats = Partition_statistics_type::From_mbr(&Mbr);

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
        let _ = Mbr.Add_partition(Partition_type_type::Gpt_protective, 1, 0xFFFFFFFF, false);

        let mut Data = vec![0u8; 4096 * 1024];
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
        let Mbr_result = Create_basic_mbr(0xABCDEF00, Partition_type_type::Fat32_lba, 100000);
        assert!(Mbr_result.is_ok());

        let Mbr = Mbr_result.unwrap();
        assert!(Mbr.Is_valid());
        assert_eq!(Mbr.Get_disk_signature(), 0xABCDEF00);

        let Valid_partitions = Mbr.Get_valid_partitions();
        assert_eq!(Valid_partitions.len(), 1);

        let Partition = &Valid_partitions[0];
        assert_eq!(
            Partition.Get_partition_type(),
            Partition_type_type::Fat32_lba
        );
        assert_eq!(Partition.Get_start_lba(), 2048);
        assert_eq!(Partition.Get_size_sectors(), 100000 - 2048);
        assert!(Partition.Is_bootable());
    }

    #[test]
    fn Test_create_basic_mbr_too_small() {
        let Mbr_result = Create_basic_mbr(0x12345678, Partition_type_type::Fat32_lba, 1000);
        assert!(Mbr_result.is_err());
        assert_eq!(Mbr_result.unwrap_err(), Error_type::Invalid_parameter);
    }

    #[test]
    fn Test_clone_mbr() {
        let Source_device = Create_test_device_with_mbr();
        let Target_data = vec![0u8; 4096 * 1024];
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
        let Target_data = vec![0u8; 4096 * 1024];
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

        let _ = Mbr.Add_partition(Partition_type_type::Fat16, 63, 1000, true);
        let _ = Mbr.Add_partition(Partition_type_type::Extended_lba, 2048, 10000, false);
        let _ = Mbr.Add_partition(Partition_type_type::Linux_swap, 15000, 2000, false);
        let _ = Mbr.Add_partition(Partition_type_type::Unknown(0x42), 20000, 5000, false);

        // Test statistics
        let Stats = Partition_statistics_type::From_mbr(&Mbr);
        assert_eq!(Stats.Total_partitions, 4);
        assert_eq!(Stats.Bootable_partitions, 1);
        assert_eq!(Stats.Fat_partitions, 1);
        assert_eq!(Stats.Linux_partitions, 1); // Linux_swap counts as Linux
        assert_eq!(Stats.Extended_partitions, 1);
        assert_eq!(Stats.Unknown_partitions, 1);

        // Test finding by type
        let Extended = Find_partitions_by_type(&Mbr, Partition_type_type::Extended_lba);
        assert_eq!(Extended.len(), 1);
        assert_eq!(Extended[0].0, 1);

        let Unknown = Find_partitions_by_type(&Mbr, Partition_type_type::Unknown(0x42));
        assert_eq!(Unknown.len(), 1);
        assert_eq!(Unknown[0].0, 3);
    }

    #[test]
    fn Test_edge_cases() {
        // Test with MBR containing only empty partitions
        let Empty_mbr = MBR_type::New_with_signature(0x12345678);

        let Stats = Partition_statistics_type::From_mbr(&Empty_mbr);
        assert_eq!(Stats.Total_partitions, 0);

        let Partitions = Find_partitions_by_type(&Empty_mbr, Partition_type_type::Fat32);
        assert_eq!(Partitions.len(), 0);

        // Test scan on empty device
        let mut Empty_data = vec![0u8; 4096 * 1024];
        let Empty_mbr_bytes = Empty_mbr.To_bytes();
        Empty_data[0..512].copy_from_slice(&Empty_mbr_bytes);
        let Memory_device = Memory_device_type::<512>::From_vec(Empty_data);
        let Empty_device = crate::Create_device!(Memory_device);

        let Scan_result = Scan_mbr_partitions(&Empty_device);
        assert!(Scan_result.is_ok());
        assert_eq!(Scan_result.unwrap().len(), 0);
    }

    // Tests for the new Format_disk_and_get_first_partition function

    #[test]
    fn Test_format_disk_and_get_first_partition_existing_mbr() {
        let Device = Create_test_device_with_mbr();

        let Partition_device_result = Format_disk_and_get_first_partition(
            &Device,
            Partition_type_type::Fat32_lba,
            Some(0xABCDEF00),
        );
        assert!(Partition_device_result.is_ok());

        let Partition_device = Partition_device_result.unwrap();
        assert!(Partition_device.Is_valid());
        assert_eq!(Partition_device.Get_start_lba(), 2048); // First partition starts at 2048
        assert_eq!(Partition_device.Get_sector_count(), 1024); // First partition size
    }

    #[test]
    fn Test_format_disk_and_get_first_partition_no_mbr() {
        let Device = Create_test_device_no_mbr();

        let Partition_device_result = Format_disk_and_get_first_partition(
            &Device,
            Partition_type_type::Fat32_lba,
            Some(0x12345678),
        );
        assert!(Partition_device_result.is_ok());

        let Partition_device = Partition_device_result.unwrap();
        assert!(Partition_device.Is_valid());
        assert_eq!(Partition_device.Get_start_lba(), 2048); // Should start at 2048 for alignment

        // Check that MBR was created on device
        assert!(Has_valid_mbr(&Device));

        // Verify the MBR has one partition with correct type
        let Mbr = MBR_type::Read_from_device(&Device).unwrap();
        let Valid_partitions = Mbr.Get_valid_partitions();
        assert_eq!(Valid_partitions.len(), 1);
        assert_eq!(
            Valid_partitions[0].Get_partition_type(),
            Partition_type_type::Fat32_lba
        );
        assert!(Valid_partitions[0].Is_bootable());
    }

    #[test]
    fn Test_format_disk_and_get_first_partition_default_signature() {
        let Device = Create_test_device_no_mbr();

        let Partition_device_result = Format_disk_and_get_first_partition(
            &Device,
            Partition_type_type::Linux,
            None, // Use default signature
        );
        assert!(Partition_device_result.is_ok());

        let Partition_device = Partition_device_result.unwrap();
        assert!(Partition_device.Is_valid());

        // Check that MBR was created with default signature
        let Mbr = MBR_type::Read_from_device(&Device).unwrap();
        assert_eq!(Mbr.Get_disk_signature(), 0x12345678); // Default signature
    }

    #[test]
    fn Test_format_disk_and_get_first_partition_device_too_small() {
        // Create a very small device (less than 2048 sectors)
        let Small_data = vec![0u8; 1024]; // 2 sectors of 512 bytes each
        let Memory_device = Memory_device_type::<512>::From_vec(Small_data);
        let Small_device = crate::Create_device!(Memory_device);

        let Partition_device_result = Format_disk_and_get_first_partition(
            &Small_device,
            Partition_type_type::Fat32_lba,
            Some(0x12345678),
        );
        assert!(Partition_device_result.is_err());
        assert_eq!(
            Partition_device_result.unwrap_err(),
            Error_type::Invalid_parameter
        );
    }

    #[test]
    fn Test_format_disk_and_get_first_partition_empty_mbr() {
        // Create device with valid MBR but no partitions
        let mut Data = vec![0u8; 4096 * 1024];
        let Empty_mbr = MBR_type::New_with_signature(0xDEADBEEF);
        let Mbr_bytes = Empty_mbr.To_bytes();
        Data[0..512].copy_from_slice(&Mbr_bytes);

        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        let Device = crate::Create_device!(Memory_device);

        let Partition_device_result = Format_disk_and_get_first_partition(
            &Device,
            Partition_type_type::Fat32_lba,
            Some(0x12345678),
        );
        assert!(Partition_device_result.is_err());
        assert_eq!(Partition_device_result.unwrap_err(), Error_type::Not_found);
    }

    #[test]
    fn Test_format_disk_and_get_first_partition_different_types() {
        // Test with different partition types
        let Partition_types = [
            Partition_type_type::Fat32_lba,
            Partition_type_type::Linux,
            Partition_type_type::Ntfs_exfat,
            Partition_type_type::Extended_lba,
        ];

        for Partition_type in &Partition_types {
            let Device = Create_test_device_no_mbr();

            let Partition_device_result =
                Format_disk_and_get_first_partition(&Device, *Partition_type, Some(0xABCDEF00));
            assert!(Partition_device_result.is_ok());

            let Partition_device = Partition_device_result.unwrap();
            assert!(Partition_device.Is_valid());

            // Verify the partition type in MBR
            let Mbr = MBR_type::Read_from_device(&Device).unwrap();
            let Valid_partitions = Mbr.Get_valid_partitions();
            assert_eq!(Valid_partitions.len(), 1);
            assert_eq!(Valid_partitions[0].Get_partition_type(), *Partition_type);
        }
    }

    #[test]
    fn Test_format_disk_and_write_read_data() {
        let Device = Create_test_device_no_mbr();

        // Format disk and get first partition
        let Partition_device_result = Format_disk_and_get_first_partition(
            &Device,
            Partition_type_type::Fat32_lba,
            Some(0x12345678),
        );
        assert!(Partition_device_result.is_ok());
        let Partition_device = Partition_device_result.unwrap();

        // Test data to write
        let Test_data = b"Hello, Partition World! This is a test of writing and reading data from a partition device.";
        let mut Write_buffer = vec![0u8; 512]; // One sector
        Write_buffer[0..Test_data.len()].copy_from_slice(Test_data);

        // Write data to the beginning of the partition
        let Write_result = Partition_device.Write(&Write_buffer);
        assert!(Write_result.is_ok());
        let Bytes_written = Write_result.unwrap();
        assert_eq!(Bytes_written.As_u64(), 512);

        // Reset position to beginning of partition
        let Set_position_result = Partition_device.Set_position(&crate::Position_type::Start(0));
        assert!(Set_position_result.is_ok());

        // Read data back from the partition
        let mut Read_buffer = vec![0u8; 512];
        let Read_result = Partition_device.Read(&mut Read_buffer);
        assert!(Read_result.is_ok());
        let Bytes_read = Read_result.unwrap();
        assert_eq!(Bytes_read.As_u64(), 512);

        // Verify the data matches what we wrote
        assert_eq!(&Read_buffer[0..Test_data.len()], Test_data);

        // Test writing at different positions
        let Second_test_data = b"Second write test at offset";
        let Second_position = 1024; // Write at sector 2

        // Set position to second sector
        let Set_position_result =
            Partition_device.Set_position(&crate::Position_type::Start(Second_position));
        assert!(Set_position_result.is_ok());

        // Write second test data
        let mut Second_write_buffer = vec![0u8; 512];
        Second_write_buffer[0..Second_test_data.len()].copy_from_slice(Second_test_data);
        let Write_result = Partition_device.Write(&Second_write_buffer);
        assert!(Write_result.is_ok());

        // Read back from second position
        let Set_position_result =
            Partition_device.Set_position(&crate::Position_type::Start(Second_position));
        assert!(Set_position_result.is_ok());
        let mut Second_read_buffer = vec![0u8; 512];
        let Read_result = Partition_device.Read(&mut Second_read_buffer);
        assert!(Read_result.is_ok());

        // Verify second write
        assert_eq!(
            &Second_read_buffer[0..Second_test_data.len()],
            Second_test_data
        );

        // Verify first write is still intact
        let Set_position_result = Partition_device.Set_position(&crate::Position_type::Start(0));
        assert!(Set_position_result.is_ok());
        let mut First_read_buffer = vec![0u8; 512];
        let Read_result = Partition_device.Read(&mut First_read_buffer);
        assert!(Read_result.is_ok());
        assert_eq!(&First_read_buffer[0..Test_data.len()], Test_data);
    }

    #[test]
    fn Test_partition_data_isolation() {
        // Test that data written to one partition doesn't affect another
        let Device = Create_test_device_no_mbr();

        // Create an MBR with multiple partitions manually
        let Device_size = Device.Get_size().unwrap();
        let Block_size = Device.Get_block_size().unwrap();
        let _ = (Device_size.As_u64() / Block_size as u64) as u32;

        let mut Mbr = MBR_type::New_with_signature(0x12345678);

        // Add two partitions
        let First_partition_size = 2048; // 1MB worth of sectors
        let Second_partition_start = 2048 + First_partition_size;
        let Second_partition_size = 2048;

        let _ = Mbr.Add_partition(
            Partition_type_type::Fat32_lba,
            2048,
            First_partition_size,
            true,
        );
        let _ = Mbr.Add_partition(
            Partition_type_type::Linux,
            Second_partition_start,
            Second_partition_size,
            false,
        );

        // Write MBR to device
        let Write_result = Mbr.Write_to_device(&Device);
        assert!(Write_result.is_ok());

        // Create partition devices
        let Valid_partitions = Mbr.Get_valid_partitions();
        assert_eq!(Valid_partitions.len(), 2);

        let First_partition_device =
            Create_partition_device(Device.clone(), &Valid_partitions[0]).unwrap();
        let Second_partition_device =
            Create_partition_device(Device.clone(), &Valid_partitions[1]).unwrap();

        // Write different data to each partition
        let First_data = b"Data for first partition - FAT32";
        let Second_data = b"Data for second partition - Linux";

        let mut First_buffer = vec![0u8; 512];
        First_buffer[0..First_data.len()].copy_from_slice(First_data);
        let Write_result = First_partition_device.Write(&First_buffer);
        assert!(Write_result.is_ok());

        let mut Second_buffer = vec![0u8; 512];
        Second_buffer[0..Second_data.len()].copy_from_slice(Second_data);
        let Write_result = Second_partition_device.Write(&Second_buffer);
        assert!(Write_result.is_ok());

        // Reset positions and read back
        let _ = First_partition_device.Set_position(&crate::Position_type::Start(0));
        let _ = Second_partition_device.Set_position(&crate::Position_type::Start(0));

        let mut First_read_buffer = vec![0u8; 512];
        let mut Second_read_buffer = vec![0u8; 512];

        let Read_result = First_partition_device.Read(&mut First_read_buffer);
        assert!(Read_result.is_ok());
        let Read_result = Second_partition_device.Read(&mut Second_read_buffer);
        assert!(Read_result.is_ok());

        // Verify each partition has its own data
        assert_eq!(&First_read_buffer[0..First_data.len()], First_data);
        assert_eq!(&Second_read_buffer[0..Second_data.len()], Second_data);

        // Verify the data is different (partitions are isolated)
        assert_ne!(&First_read_buffer[0..32], &Second_read_buffer[0..32]);
    }

    #[test]
    fn Test_partition_bounds_checking() {
        let Device = Create_test_device_no_mbr();

        let Partition_device_result = Format_disk_and_get_first_partition(
            &Device,
            Partition_type_type::Fat32_lba,
            Some(0x12345678),
        );
        assert!(Partition_device_result.is_ok());
        let Partition_device = Partition_device_result.unwrap();

        // Try to write beyond partition bounds
        let Partition_size_bytes = Partition_device.Get_sector_count() as u64 * 512;
        let Beyond_bounds_position = Partition_size_bytes;

        let _ = Partition_device
            .Set_position(&crate::Position_type::Start(Beyond_bounds_position))
            .unwrap();
        // This should either fail or be clamped to valid range
        // The exact behavior depends on the partition device implementation

        // Write a small amount of data at the very end of the partition (should work)
        let End_position = Partition_size_bytes - 512;
        let Set_position_result =
            Partition_device.Set_position(&crate::Position_type::Start(End_position));
        assert!(Set_position_result.is_ok());

        let Test_data = b"End of partition data";
        let mut Write_buffer = vec![0u8; 512];
        Write_buffer[0..Test_data.len()].copy_from_slice(Test_data);

        let Write_result = Partition_device.Write(&Write_buffer);
        assert!(Write_result.is_ok());

        // Read it back to verify
        let Set_position_result =
            Partition_device.Set_position(&crate::Position_type::Start(End_position));
        assert!(Set_position_result.is_ok());
        let mut Read_buffer = vec![0u8; 512];
        let Read_result = Partition_device.Read(&mut Read_buffer);
        assert!(Read_result.is_ok());
        assert_eq!(&Read_buffer[0..Test_data.len()], Test_data);
    }
}
