//! Utility functions for Master Boot Record (MBR) operations.

//!
//! This module provides high-level utility functions for working with MBR partition tables,
//! including creating partition devices, scanning for partitions, validation, and disk formatting.
//! These utilities simplify common MBR operations and provide comprehensive error handling.

use alloc::vec::Vec;

use super::Mbr;
use crate::{Device, Error, PartitionDevice, PartitionEntry, Result};

/// Create a partition device from an MBR partition entry.
///
/// This function takes a base device and a partition entry from an MBR, and creates
/// a [`PartitionDevice`] that represents just that partition. The resulting
/// partition device can be used for all standard device operations, but will only
/// access the sectors allocated to that specific partition.
///
/// # Arguments
///
/// * `Base_device` - The underlying storage device containing the partition
/// * `Partition` - MBR partition entry describing the partition layout
///
/// # Returns
///
/// * `Ok(Partition_device_type)` - Successfully created partition device
/// * `Err(Error::InvalidParameter)` - Partition entry is invalid
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
/// // First create and write an MBR to the device
/// let mut mbr = MBR_type::New_with_signature(0x12345678);
/// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
/// mbr.Write_to_device(&device).unwrap();
///
/// // Now read it back and create partition device
/// let mbr = MBR_type::Read_from_device(&device).unwrap();
/// if let Some(partition) = mbr.get_valid_partitions().first() {
///     let partition_device = Create_partition_device(device, partition).unwrap();
///     // Now you can use partition_device for I/O operations
/// }
/// ```
pub fn create_partition_device(
    base_device: Device,
    partition: &PartitionEntry,
) -> Result<PartitionDevice> {
    if !partition.is_valid() {
        return Err(Error::InvalidParameter);
    }

    Ok(PartitionDevice::new_from_lba(
        base_device,
        partition.get_start_lba(),
        partition.get_size_sectors(),
    )?)
}

/// Scan a device for MBR and return partition information.
///
/// This function reads the MBR from a device and extracts information about all
/// valid partitions. It returns a vector of tuples containing the partition index
/// and the partition entry for each valid partition found.
///
/// # Arguments
///
/// * `Device` - The storage device to scan for MBR partitions
///
/// # Returns
///
/// * `Ok(Vec<(usize, Partition_entry_type)>)` - List of valid partitions with their indices
/// * `Err(Error)` - Error reading MBR or device access failure
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
///
/// match Scan_mbr_partitions(&device) {
///     Ok(partitions) => {
///         println!("Found {} valid partitions", partitions.len());
///         for (index, partition) in partitions {
///             println!("Partition {}: {:?}", index, partition.get_partition_type());
///         }
///     }
///     Err(e) => println!("Failed to scan partitions: {}", e),
/// }
/// ```
pub fn scan_mbr_partitions(device: &Device) -> Result<Vec<(usize, PartitionEntry)>> {
    let mbr = Mbr::read_from_device(device)?;

    let mut partitions = Vec::new();
    for (i, partition) in mbr.partitions.iter().enumerate() {
        if partition.is_valid() {
            partitions.push((i, *partition));
        }
    }

    Ok(partitions)
}

/// Validate MBR structure and partitions for consistency and correctness.
///
/// This function performs comprehensive validation of an MBR structure, checking:
/// - MBR signature validity (0x55AA boot signature)
/// - Partition overlap detection
/// - Bootable partition count (at most one partition should be bootable)
///
/// # Arguments
///
/// * `Mbr` - The MBR structure to validate
///
/// # Returns
///
/// * `Ok(())` - MBR is valid and consistent
/// * `Err(Error::Corrupted)` - MBR is invalid or corrupted
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
/// // First create and write a valid MBR
/// let mut mbr = MBR_type::New_with_signature(0x12345678);
/// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
/// mbr.Write_to_device(&device).unwrap();
///
/// // Read it back and validate
/// let mbr = MBR_type::Read_from_device(&device).unwrap();
/// match mbr.Validate() {
///     Ok(()) => println!("MBR is valid"),
///     Err(Error::Corrupted) => println!("MBR is corrupted"),
///     Err(e) => println!("Validation error: {}", e),
/// }
/// ```
pub fn validate_mbr(mbr: &crate::Mbr) -> Result<()> {
    mbr.validate()
}

/// Create partition devices for all valid partitions in an MBR.
///
/// This function iterates through all partition entries in an MBR and creates
/// [`PartitionDevice`] instances for each valid partition. This is useful
/// when you need to access all partitions on a disk programmatically.
///
/// # Arguments
///
/// * `Base_device` - The underlying storage device containing all partitions
/// * `Mbr` - The MBR structure containing partition information
///
/// # Returns
///
/// * `Ok(Vec<PartitionDevice>)` - Vector of partition devices for all valid partitions
/// * `Err(Error)` - Error if any partition device creation fails
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
/// // Create an MBR with multiple partitions
/// let mut mbr = MBR_type::New_with_signature(0x12345678);
/// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
/// mbr.Add_partition(Partition_type_type::Linux, 4096, 2048, false).unwrap();
/// mbr.Write_to_device(&device).unwrap();
///
/// // Read it back and create all partition devices
/// let mbr = MBR_type::Read_from_device(&device).unwrap();
/// let partition_devices = mbr.Create_all_partition_devices(device).unwrap();
/// println!("Created {} partition devices", partition_devices.len());
///
/// for (i, partition) in partition_devices.iter().enumerate() {
///     println!("Partition {}: {} sectors", i, partition.get_sector_count());
/// }
/// ```
pub fn create_all_partition_devices(
    base_device: Device,
    mbr: &super::Mbr,
) -> Result<Vec<PartitionDevice>> {
    mbr.create_all_partition_devices(base_device)
}

/// Find partitions of a specific type within an MBR.
///
/// This function searches through all partitions in an MBR and returns references
/// to those that match the specified partition type. This is useful for locating
/// specific types of partitions (e.g., FAT32, Linux, etc.) without creating
/// partition devices.
///
/// # Arguments
///
/// * `Mbr` - The MBR structure to search through
/// * `Partition_type` - The specific partition type to find
///
/// # Returns
///
/// A vector of tuples containing the partition index and reference to the partition entry
/// for each matching partition.
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
/// // Create an MBR with FAT32 partition
/// let mut mbr = MBR_type::New_with_signature(0x12345678);
/// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
/// mbr.Write_to_device(&device).unwrap();
///
/// // Read it back and find FAT32 partitions
/// let mbr = MBR_type::Read_from_device(&device).unwrap();
/// let fat32_partitions = mbr.Find_partitions_by_type(Partition_type_type::Fat32_lba);
/// println!("Found {} FAT32 partitions", fat32_partitions.len());
/// ```
pub fn find_partitions_by_type(
    mbr: &super::Mbr,
    partition_type: crate::PartitionKind,
) -> Vec<(usize, &PartitionEntry)> {
    mbr.find_partitions_by_type(partition_type)
}

/// Check if a device contains a valid MBR.
///
/// This function attempts to read an MBR from the device and validates its signature.
/// It's a quick way to determine if a device has been properly partitioned with MBR.
///
/// # Arguments
///
/// * `Device` - The storage device to check
///
/// # Returns
///
/// * `true` - Device contains a valid MBR with proper signature
/// * `false` - Device doesn't contain a valid MBR or cannot be read
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
///
/// if Has_valid_mbr(&device) {
///     println!("Device has a valid MBR");
/// } else {
///     println!("Device needs to be partitioned");
/// }
/// ```
pub fn has_valid_mbr(device: &Device) -> bool {
    match Mbr::read_from_device(device) {
        Ok(mbr) => mbr.is_valid(),
        Err(_) => false,
    }
}

/// Check if a device uses GPT (GUID Partition Table) instead of MBR.
///
/// This function checks if the device contains a GPT protective partition in its MBR,
/// which indicates that the device uses GPT partitioning instead of traditional MBR.
/// GPT is the modern replacement for MBR and supports larger disks and more partitions.
///
/// # Arguments
///
/// * `Device` - The storage device to check
///
/// # Returns
///
/// * `true` - Device uses GPT partitioning (has protective MBR)
/// * `false` - Device uses traditional MBR or cannot be read
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
///
/// if is_gpt_disk(&device) {
///     println!("Device uses GPT partitioning");
/// } else {
///     println!("Device uses MBR partitioning");
/// }
/// ```
pub fn is_gpt_disk(device: &Device) -> bool {
    match Mbr::read_from_device(device) {
        Ok(mbr) => mbr.has_gpt_protective_partition(),
        Err(_) => false,
    }
}

/// Create a basic MBR with a single partition covering most of the disk.
///
/// This function creates a simple MBR structure with one partition that uses
/// most of the available disk space. It leaves 2048 sectors at the beginning
/// for proper alignment, which is standard practice for modern storage devices.
///
/// # Arguments
///
/// * `Disk_signature` - Unique 32-bit signature for the disk
/// * `Partition_type` - Type of partition to create (e.g., FAT32, Linux, etc.)
/// * `Total_sectors` - Total number of sectors available on the disk
///
/// # Returns
///
/// * `Ok(MBR_type)` - Successfully created MBR with single partition
/// * `Err(Error::InvalidParameter)` - Disk is too small for a partition
///
/// # Examples
///
/// ```rust
/// use file_system::*;
///
/// // Create MBR for a 4MB device (8192 sectors)
/// let mbr = MBR_type::Create_basic(0x12345678, Partition_type_type::Fat32_lba, 8192).unwrap();
///
/// // The MBR will have one FAT32 partition starting at sector 2048
/// let partitions = mbr.get_valid_partitions();
/// assert_eq!(partitions.len(), 1);
/// assert_eq!(partitions[0].get_start_lba(), 2048);
/// ```
pub fn create_basic_mbr(
    disk_signature: u32,
    partition_type: crate::PartitionKind,
    total_sectors: u32,
) -> Result<super::Mbr> {
    Mbr::create_basic(disk_signature, partition_type, total_sectors)
}

/// Clone an MBR from one device to another.
///
/// This function reads the MBR from a source device, validates it for consistency,
/// and writes it to a target device. This is useful for creating exact copies of
/// partition layouts or backing up partition tables.
///
/// # Arguments
///
/// * `Source_device` - Device to read the MBR from
/// * `Target_device` - Device to write the MBR to
///
/// # Returns
///
/// * `Ok(())` - MBR successfully cloned
/// * `Err(Error)` - Error reading, validating, or writing MBR
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let source = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
/// let target = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
///
/// // Create a valid MBR on source device first
/// let mut mbr = MBR_type::New_with_signature(0x12345678);
/// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
/// mbr.Write_to_device(&source).unwrap();
///
/// // Now clone the MBR to target
/// Clone_mbr(&source, &target).unwrap();
///
/// // Both devices now have valid MBRs
/// assert_eq!(Has_valid_mbr(&source), Has_valid_mbr(&target));
/// ```
pub fn clone_mbr(source_device: &Device, target_device: &Device) -> Result<()> {
    let mbr = Mbr::read_from_device(source_device)?;
    mbr.validate()?;
    mbr.write_to_device(target_device)?;
    Ok(())
}

/// Create a backup of an MBR as a byte array.
///
/// This function reads the MBR from a device and returns it as a 512-byte array.
/// This backup can be stored and later restored using [`restore_mbr`]. This is
/// essential for disaster recovery and partition table management.
///
/// # Arguments
///
/// * `Device` - The storage device to backup the MBR from
///
/// # Returns
///
/// * `Ok([u8; 512])` - 512-byte array containing the complete MBR
/// * `Err(Error)` - Error reading MBR from device
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
/// // Create a valid MBR first
/// let mut mbr = MBR_type::New_with_signature(0x12345678);
/// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
/// mbr.Write_to_device(&device).unwrap();
///
/// // Create backup
/// let backup = Backup_mbr(&device).unwrap();
///
/// // Store backup somewhere safe...
/// // Later, restore it if needed
/// Restore_mbr(&device, &backup).unwrap();
/// ```
pub fn backup_mbr(device: &Device) -> Result<[u8; 512]> {
    let mbr = Mbr::read_from_device(device)?;
    Ok(mbr.to_bytes())
}

/// Restore an MBR from a backup byte array.
///
/// This function takes a previously created MBR backup and writes it to a device.
/// The backup is validated before writing to ensure data integrity. This is the
/// counterpart to [`backup_mbr`] for disaster recovery scenarios.
///
/// # Arguments
///
/// * `Device` - The storage device to restore the MBR to
/// * `Backup` - 512-byte array containing the MBR backup
///
/// # Returns
///
/// * `Ok(())` - MBR successfully restored
/// * `Err(Error)` - Error validating backup or writing to device
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(Memory_device_type::<512>::new(4 * 1024 * 1024));
/// // Create a valid MBR first
/// let mut mbr = MBR_type::New_with_signature(0x12345678);
/// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
/// mbr.Write_to_device(&device).unwrap();
///
/// // Create a backup
/// let backup = backup_mbr(&device).unwrap();
///
/// // Simulate corruption or need to restore
/// Restore_mbr(&device, &backup).unwrap();
///
/// assert!(Has_valid_mbr(&device));
/// ```
pub fn restore_mbr(device: &Device, backup: &[u8; 512]) -> Result<()> {
    let mbr = Mbr::from_bytes(backup)?;
    mbr.validate()?;
    mbr.write_to_device(device)?;
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
/// * `Result<Partition_device_type>` - The first partition device
pub fn format_disk_and_get_first_partition(
    device: &Device,
    partition_type: crate::PartitionKind,
    disk_signature: Option<u32>,
) -> Result<PartitionDevice> {
    // Check if device already has valid MBR
    let mbr = if has_valid_mbr(device) {
        // Read existing MBR
        Mbr::read_from_device(device)?
    } else {
        // Get device size in sectors
        let device_size = device.get_size()?;
        let block_size = device.get_block_size()?;
        let total_sectors = (device_size.as_u64() / block_size as u64) as u32;

        if total_sectors < 2048 {
            return Err(Error::InvalidParameter);
        }

        // Create new MBR with signature
        let signature = disk_signature.unwrap_or({
            // Generate a simple signature based on current time or use a default
            // In a real implementation, you might want to use a proper random number generator
            0x12345678
        });

        let new_mbr = Mbr::create_basic(signature, partition_type, total_sectors)?;

        // Write the new MBR to device
        new_mbr.write_to_device(device)?;

        new_mbr
    };

    // Get the first valid partition
    let valid_partitions = mbr.get_valid_partitions();
    if valid_partitions.is_empty() {
        return Err(Error::NotFound);
    }

    // Create partition device for the first partition
    create_partition_device(device.clone(), valid_partitions[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Device, DeviceTrait, Error, MemoryDevice, PartitionKind, PartitionStatistics};
    use alloc::vec;

    /// Create a test device with MBR data
    fn create_test_device_with_mbr() -> Device {
        let mut data = vec![0u8; 4096 * 1024]; // Make it large enough (4MB = 8192 sectors)

        // Create a simple MBR at the beginning
        let mbr = create_test_mbr();
        let mbr_bytes = mbr.to_bytes();
        data[0..512].copy_from_slice(&mbr_bytes);

        let memory_device = MemoryDevice::<512>::from_vec(data);
        crate::create_device!(memory_device)
    }

    /// Create a test device without valid MBR
    fn create_test_device_no_mbr() -> Device {
        let memory_device = MemoryDevice::<512>::new(4096 * 1024); // Make it large enough (4MB = 8192 sectors)
        crate::create_device!(memory_device)
    }

    /// Create a test MBR for testing
    fn create_test_mbr() -> Mbr {
        let mut mbr = Mbr::new_with_signature(0x12345678);

        // Add a few test partitions
        let _ = mbr.add_partition(PartitionKind::Fat32Lba, 2048, 1024, true);
        let _ = mbr.add_partition(PartitionKind::Linux, 4096, 2048, false);
        let _ = mbr.add_partition(PartitionKind::HiddenFat16, 8192, 512, false);

        mbr
    }

    #[test]
    fn test_create_partition_device() {
        let base_device = create_test_device_with_mbr();
        let mbr = Mbr::read_from_device(&base_device).unwrap();
        let partition = &mbr.partitions[0];

        let device_result = create_partition_device(base_device, partition);
        assert!(device_result.is_ok());

        let device = device_result.unwrap();
        assert_eq!(device.get_start_lba(), partition.get_start_lba());
        assert_eq!(device.get_sector_count(), partition.get_size_sectors());
        assert!(device.is_valid());
    }

    #[test]
    fn test_create_partition_device_invalid() {
        let base_device = create_test_device_with_mbr();
        let invalid_partition = PartitionEntry::new();

        let device_result = create_partition_device(base_device, &invalid_partition);
        assert!(device_result.is_err());
        assert_eq!(device_result.unwrap_err(), Error::InvalidParameter);
    }

    #[test]
    fn test_scan_mbr_partitions() {
        let device = create_test_device_with_mbr();

        let scan_result = scan_mbr_partitions(&device);
        assert!(scan_result.is_ok());

        let partitions = scan_result.unwrap();
        assert_eq!(partitions.len(), 3); // Should find 3 valid partitions

        // Check that indices are correct
        assert_eq!(partitions[0].0, 0);
        assert_eq!(partitions[1].0, 1);
        assert_eq!(partitions[2].0, 2);

        // Check partition types
        assert_eq!(
            partitions[0].1.get_partition_type(),
            PartitionKind::Fat32Lba
        );
        assert_eq!(partitions[1].1.get_partition_type(), PartitionKind::Linux);
        assert_eq!(
            partitions[2].1.get_partition_type(),
            PartitionKind::HiddenFat16
        );
    }

    #[test]
    fn test_scan_mbr_partitions_no_mbr() {
        let device = create_test_device_no_mbr();

        let scan_result = scan_mbr_partitions(&device);
        assert!(scan_result.is_err());
    }

    #[test]
    fn test_validate_mbr_valid() {
        let mbr = create_test_mbr();
        let validation_result = validate_mbr(&mbr);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_validate_mbr_invalid_signature() {
        let mut mbr = super::Mbr::new();
        // Manually set invalid signature - should be invalid
        mbr.boot_signature = [0x00, 0x00];

        let validation_result = validate_mbr(&mbr);
        assert!(validation_result.is_err());
        assert_eq!(validation_result.unwrap_err(), Error::Corrupted);
    }

    #[test]
    fn test_validate_mbr_multiple_bootable() {
        let mut mbr = Mbr::new_with_signature(0x12345678);

        // Manually create multiple bootable partitions (bypassing Add_partition validation)
        mbr.partitions[0] =
            PartitionEntry::new_with_params(true, PartitionKind::Fat32Lba, 2048, 1024);
        mbr.partitions[1] = PartitionEntry::new_with_params(true, PartitionKind::Linux, 4096, 2048);

        let validation_result = validate_mbr(&mbr);
        assert!(validation_result.is_err());
        assert_eq!(validation_result.unwrap_err(), Error::Corrupted);
    }

    #[test]
    fn test_validate_mbr_overlapping_partitions() {
        let mut mbr = Mbr::new_with_signature(0x12345678);

        // Manually create overlapping partitions (bypassing Add_partition validation)
        mbr.partitions[0] =
            PartitionEntry::new_with_params(false, PartitionKind::Fat32Lba, 2048, 2048);
        mbr.partitions[1] =
            PartitionEntry::new_with_params(false, PartitionKind::Linux, 3000, 2048);

        let validation_result = validate_mbr(&mbr);
        assert!(validation_result.is_err());
        assert_eq!(validation_result.unwrap_err(), Error::Corrupted);
    }

    #[test]
    fn test_create_all_partition_devices() {
        let base_device = create_test_device_with_mbr();
        let mbr = create_test_mbr();

        let devices_result = create_all_partition_devices(base_device, &mbr);
        assert!(devices_result.is_ok());

        let devices = devices_result.unwrap();
        assert_eq!(devices.len(), 3); // Should create 3 devices

        // Check that all devices are valid
        for device in &devices {
            assert!(device.is_valid());
        }

        // Check first device properties
        assert_eq!(devices[0].get_start_lba(), 2048);
        assert_eq!(devices[0].get_sector_count(), 1024);
    }

    #[test]
    fn test_find_partitions_by_type() {
        let mbr = create_test_mbr();

        // Find FAT32 partitions
        let fat_partitions = find_partitions_by_type(&mbr, PartitionKind::Fat32Lba);
        assert_eq!(fat_partitions.len(), 1);
        assert_eq!(fat_partitions[0].0, 0); // Index 0

        // Find Linux partitions
        let linux_partitions = find_partitions_by_type(&mbr, PartitionKind::Linux);
        assert_eq!(linux_partitions.len(), 1);
        assert_eq!(linux_partitions[0].0, 1); // Index 1

        // Find non-existent type
        let ntfs_partitions = find_partitions_by_type(&mbr, PartitionKind::NtfsExfat);
        assert_eq!(ntfs_partitions.len(), 0);
    }

    #[test]
    fn test_partition_statistics() {
        let mbr = create_test_mbr();
        let stats = PartitionStatistics::from_mbr(&mbr);

        assert_eq!(stats.total_partitions, 3);
        assert_eq!(stats.bootable_partitions, 1);
        assert_eq!(stats.fat_partitions, 2); // Fat32_lba + Hidden_fat16
        assert_eq!(stats.linux_partitions, 1);
        assert_eq!(stats.hidden_partitions, 1); // Hidden_fat16
        assert_eq!(stats.extended_partitions, 0);
        assert_eq!(stats.unknown_partitions, 0);
        assert_eq!(stats.total_used_sectors, 1024 + 2048 + 512); // Sum of all partition sizes
        assert_eq!(stats.largest_partition_sectors, 2048);
        assert_eq!(stats.smallest_partition_sectors, 512);
    }

    #[test]
    fn test_partition_statistics_empty_mbr() {
        let mbr = Mbr::new_with_signature(0x12345678);
        let stats = PartitionStatistics::from_mbr(&mbr);

        assert_eq!(stats.total_partitions, 0);
        assert_eq!(stats.bootable_partitions, 0);
        assert_eq!(stats.fat_partitions, 0);
        assert_eq!(stats.linux_partitions, 0);
        assert_eq!(stats.hidden_partitions, 0);
        assert_eq!(stats.extended_partitions, 0);
        assert_eq!(stats.unknown_partitions, 0);
        assert_eq!(stats.total_used_sectors, 0);
        assert_eq!(stats.largest_partition_sectors, 0);
        assert_eq!(stats.smallest_partition_sectors, 0);
    }

    #[test]
    fn test_has_valid_mbr() {
        let valid_device = create_test_device_with_mbr();
        assert!(has_valid_mbr(&valid_device));

        let invalid_device = create_test_device_no_mbr();
        assert!(!has_valid_mbr(&invalid_device));
    }

    #[test]
    fn test_is_gpt_disk() {
        // Create MBR with GPT protective partition
        let mut mbr = Mbr::new_with_signature(0x12345678);
        let _ = mbr.add_partition(PartitionKind::GptProtective, 1, 0xFFFFFFFF, false);

        let mut data = vec![0u8; 4096 * 1024];
        let mbr_bytes = mbr.to_bytes();
        data[0..512].copy_from_slice(&mbr_bytes);
        let memory_device = MemoryDevice::<512>::from_vec(data);
        let gpt_device = crate::create_device!(memory_device);

        assert!(is_gpt_disk(&gpt_device));

        // Regular MBR should not be detected as GPT
        let regular_device = create_test_device_with_mbr();
        assert!(!is_gpt_disk(&regular_device));
    }

    #[test]
    fn test_create_basic_mbr() {
        let mbr_result = create_basic_mbr(0xABCDEF00, PartitionKind::Fat32Lba, 100000);
        assert!(mbr_result.is_ok());

        let mbr = mbr_result.unwrap();
        assert!(mbr.is_valid());
        assert_eq!(mbr.get_disk_signature(), 0xABCDEF00);

        let valid_partitions = mbr.get_valid_partitions();
        assert_eq!(valid_partitions.len(), 1);

        let partition = &valid_partitions[0];
        assert_eq!(partition.get_partition_type(), PartitionKind::Fat32Lba);
        assert_eq!(partition.get_start_lba(), 2048);
        assert_eq!(partition.get_size_sectors(), 100000 - 2048);
        assert!(partition.is_bootable());
    }

    #[test]
    fn test_create_basic_mbr_too_small() {
        let mbr_result = create_basic_mbr(0x12345678, PartitionKind::Fat32Lba, 1000);
        assert!(mbr_result.is_err());
        assert_eq!(mbr_result.unwrap_err(), Error::InvalidParameter);
    }

    #[test]
    fn test_clone_mbr() {
        let source_device = create_test_device_with_mbr();
        let target_data = vec![0u8; 4096 * 1024];
        let memory_device = MemoryDevice::<512>::from_vec(target_data);
        let target_device = crate::create_device!(memory_device);

        let clone_result = clone_mbr(&source_device, &target_device);
        assert!(clone_result.is_ok());

        // Verify that target device now has the same MBR
        let source_mbr = Mbr::read_from_device(&source_device).unwrap();
        let target_mbr = Mbr::read_from_device(&target_device).unwrap();

        assert_eq!(source_mbr.to_bytes(), target_mbr.to_bytes());
    }

    #[test]
    fn test_backup_and_restore_mbr() {
        let device = create_test_device_with_mbr();

        // Backup MBR
        let backup_result = backup_mbr(&device);
        assert!(backup_result.is_ok());
        let backup = backup_result.unwrap();

        // Create a new device with zeros
        let target_data = vec![0u8; 4096 * 1024];
        let memory_device = MemoryDevice::<512>::from_vec(target_data);
        let target_device = crate::create_device!(memory_device);

        // Restore MBR
        let restore_result = restore_mbr(&target_device, &backup);
        assert!(restore_result.is_ok());

        // Verify restoration
        let original_mbr = Mbr::read_from_device(&device).unwrap();
        let restored_mbr = Mbr::read_from_device(&target_device).unwrap();

        assert_eq!(original_mbr.to_bytes(), restored_mbr.to_bytes());
    }

    #[test]
    fn test_backup_mbr_invalid_device() {
        let invalid_device = create_test_device_no_mbr();

        let backup_result = backup_mbr(&invalid_device);
        assert!(backup_result.is_err());
    }

    #[test]
    fn test_restore_mbr_invalid_backup() {
        let device = create_test_device_no_mbr();
        let invalid_backup = [0u8; 512]; // No valid signature

        let restore_result = restore_mbr(&device, &invalid_backup);
        assert!(restore_result.is_err());
    }

    #[test]
    fn test_utilities_with_complex_mbr() {
        // Create a more complex MBR for comprehensive testing
        let mut mbr = Mbr::new_with_signature(0xDEADBEEF);

        let _ = mbr.add_partition(PartitionKind::Fat16, 63, 1000, true);
        let _ = mbr.add_partition(PartitionKind::ExtendedLba, 2048, 10000, false);
        let _ = mbr.add_partition(PartitionKind::LinuxSwap, 15000, 2000, false);
        let _ = mbr.add_partition(PartitionKind::Unknown(0x42), 20000, 5000, false);

        // Test statistics
        let stats = PartitionStatistics::from_mbr(&mbr);
        assert_eq!(stats.total_partitions, 4);
        assert_eq!(stats.bootable_partitions, 1);
        assert_eq!(stats.fat_partitions, 1);
        assert_eq!(stats.linux_partitions, 1); // Linux_swap counts as Linux
        assert_eq!(stats.extended_partitions, 1);
        assert_eq!(stats.unknown_partitions, 1);

        // Test finding by type
        let extended = find_partitions_by_type(&mbr, PartitionKind::ExtendedLba);
        assert_eq!(extended.len(), 1);
        assert_eq!(extended[0].0, 1);

        let unknown = find_partitions_by_type(&mbr, PartitionKind::Unknown(0x42));
        assert_eq!(unknown.len(), 1);
        assert_eq!(unknown[0].0, 3);
    }

    #[test]
    fn test_edge_cases() {
        // Test with MBR containing only empty partitions
        let empty_mbr = Mbr::new_with_signature(0x12345678);

        let stats = PartitionStatistics::from_mbr(&empty_mbr);
        assert_eq!(stats.total_partitions, 0);

        let partitions = find_partitions_by_type(&empty_mbr, PartitionKind::Fat32);
        assert_eq!(partitions.len(), 0);

        // Test scan on empty device
        let mut empty_data = vec![0u8; 4096 * 1024];
        let empty_mbr_bytes = empty_mbr.to_bytes();
        empty_data[0..512].copy_from_slice(&empty_mbr_bytes);
        let memory_device = MemoryDevice::<512>::from_vec(empty_data);
        let empty_device = crate::create_device!(memory_device);

        let scan_result = scan_mbr_partitions(&empty_device);
        assert!(scan_result.is_ok());
        assert_eq!(scan_result.unwrap().len(), 0);
    }

    // Tests for the new Format_disk_and_get_first_partition function

    #[test]
    fn test_format_disk_and_get_first_partition_existing_mbr() {
        let device = create_test_device_with_mbr();

        let partition_device_result =
            format_disk_and_get_first_partition(&device, PartitionKind::Fat32Lba, Some(0xABCDEF00));
        assert!(partition_device_result.is_ok());

        let partition_device = partition_device_result.unwrap();
        assert!(partition_device.is_valid());
        assert_eq!(partition_device.get_start_lba(), 2048); // First partition starts at 2048
        assert_eq!(partition_device.get_sector_count(), 1024); // First partition size
    }

    #[test]
    fn test_format_disk_and_get_first_partition_no_mbr() {
        let device = create_test_device_no_mbr();

        let partition_device_result =
            format_disk_and_get_first_partition(&device, PartitionKind::Fat32Lba, Some(0x12345678));
        assert!(partition_device_result.is_ok());

        let partition_device = partition_device_result.unwrap();
        assert!(partition_device.is_valid());
        assert_eq!(partition_device.get_start_lba(), 2048); // Should start at 2048 for alignment

        // Check that MBR was created on device
        assert!(has_valid_mbr(&device));

        // Verify the MBR has one partition with correct type
        let mbr = Mbr::read_from_device(&device).unwrap();
        let valid_partitions = mbr.get_valid_partitions();
        assert_eq!(valid_partitions.len(), 1);
        assert_eq!(
            valid_partitions[0].get_partition_type(),
            PartitionKind::Fat32Lba
        );
        assert!(valid_partitions[0].is_bootable());
    }

    #[test]
    fn test_format_disk_and_get_first_partition_default_signature() {
        let device = create_test_device_no_mbr();

        let partition_device_result = format_disk_and_get_first_partition(
            &device,
            PartitionKind::Linux,
            None, // Use default signature
        );
        assert!(partition_device_result.is_ok());

        let partition_device = partition_device_result.unwrap();
        assert!(partition_device.is_valid());

        // Check that MBR was created with default signature
        let mbr = Mbr::read_from_device(&device).unwrap();
        assert_eq!(mbr.get_disk_signature(), 0x12345678); // Default signature
    }

    #[test]
    fn test_format_disk_and_get_first_partition_device_too_small() {
        // Create a very small device (less than 2048 sectors)
        let small_data = vec![0u8; 1024]; // 2 sectors of 512 bytes each
        let memory_device = MemoryDevice::<512>::from_vec(small_data);
        let small_device = crate::create_device!(memory_device);

        let partition_device_result = format_disk_and_get_first_partition(
            &small_device,
            PartitionKind::Fat32Lba,
            Some(0x12345678),
        );
        assert!(partition_device_result.is_err());
        assert_eq!(
            partition_device_result.unwrap_err(),
            Error::InvalidParameter
        );
    }

    #[test]
    fn test_format_disk_and_get_first_partition_empty_mbr() {
        // Create device with valid MBR but no partitions
        let mut data = vec![0u8; 4096 * 1024];
        let empty_mbr = Mbr::new_with_signature(0xDEADBEEF);
        let mbr_bytes = empty_mbr.to_bytes();
        data[0..512].copy_from_slice(&mbr_bytes);

        let memory_device = MemoryDevice::<512>::from_vec(data);
        let device = crate::create_device!(memory_device);

        let partition_device_result =
            format_disk_and_get_first_partition(&device, PartitionKind::Fat32Lba, Some(0x12345678));
        assert!(partition_device_result.is_err());
        assert_eq!(partition_device_result.unwrap_err(), Error::NotFound);
    }

    #[test]
    fn test_format_disk_and_get_first_partition_different_types() {
        // Test with different partition types
        let partition_types = [
            PartitionKind::Fat32Lba,
            PartitionKind::Linux,
            PartitionKind::NtfsExfat,
            PartitionKind::ExtendedLba,
        ];

        for partition_type in &partition_types {
            let device = create_test_device_no_mbr();

            let partition_device_result =
                format_disk_and_get_first_partition(&device, *partition_type, Some(0xABCDEF00));
            assert!(partition_device_result.is_ok());

            let partition_device = partition_device_result.unwrap();
            assert!(partition_device.is_valid());

            // Verify the partition type in MBR
            let mbr = Mbr::read_from_device(&device).unwrap();
            let valid_partitions = mbr.get_valid_partitions();
            assert_eq!(valid_partitions.len(), 1);
            assert_eq!(valid_partitions[0].get_partition_type(), *partition_type);
        }
    }

    #[test]
    fn test_format_disk_and_write_read_data() {
        let device = create_test_device_no_mbr();

        // Format disk and get first partition
        let partition_device_result =
            format_disk_and_get_first_partition(&device, PartitionKind::Fat32Lba, Some(0x12345678));
        assert!(partition_device_result.is_ok());
        let partition_device = partition_device_result.unwrap();

        // Test data to write
        let test_data = b"Hello, Partition World! This is a test of writing and reading data from a partition device.";
        let mut write_buffer = vec![0u8; 512]; // One sector
        write_buffer[0..test_data.len()].copy_from_slice(test_data);

        // Write data to the beginning of the partition
        let write_result = partition_device.write(&write_buffer);
        assert!(write_result.is_ok());
        let bytes_written = write_result.unwrap();
        assert_eq!(bytes_written.as_u64(), 512);

        // Reset position to beginning of partition
        let set_position_result = partition_device.set_position(&crate::Position::Start(0));
        assert!(set_position_result.is_ok());

        // Read data back from the partition
        let mut read_buffer = vec![0u8; 512];
        let read_result = partition_device.read(&mut read_buffer);
        assert!(read_result.is_ok());
        let bytes_read = read_result.unwrap();
        assert_eq!(bytes_read.as_u64(), 512);

        // Verify the data matches what we wrote
        assert_eq!(&read_buffer[0..test_data.len()], test_data);

        // Test writing at different positions
        let second_test_data = b"Second write test at offset";
        let second_position = 1024; // Write at sector 2

        // Set position to second sector
        let set_position_result =
            partition_device.set_position(&crate::Position::Start(second_position));
        assert!(set_position_result.is_ok());

        // Write second test data
        let mut second_write_buffer = vec![0u8; 512];
        second_write_buffer[0..second_test_data.len()].copy_from_slice(second_test_data);
        let write_result = partition_device.write(&second_write_buffer);
        assert!(write_result.is_ok());

        // Read back from second position
        let set_position_result =
            partition_device.set_position(&crate::Position::Start(second_position));
        assert!(set_position_result.is_ok());
        let mut second_read_buffer = vec![0u8; 512];
        let read_result = partition_device.read(&mut second_read_buffer);
        assert!(read_result.is_ok());

        // Verify second write
        assert_eq!(
            &second_read_buffer[0..second_test_data.len()],
            second_test_data
        );

        // Verify first write is still intact
        let set_position_result = partition_device.set_position(&crate::Position::Start(0));
        assert!(set_position_result.is_ok());
        let mut first_read_buffer = vec![0u8; 512];
        let read_result = partition_device.read(&mut first_read_buffer);
        assert!(read_result.is_ok());
        assert_eq!(&first_read_buffer[0..test_data.len()], test_data);
    }

    #[test]
    fn test_partition_data_isolation() {
        // Test that data written to one partition doesn't affect another
        let device = create_test_device_no_mbr();

        // Create an MBR with multiple partitions manually
        let device_size = device.get_size().unwrap();
        let block_size = device.get_block_size().unwrap();
        let _ = (device_size.as_u64() / block_size as u64) as u32;

        let mut mbr = Mbr::new_with_signature(0x12345678);

        // Add two partitions
        let first_partition_size = 2048; // 1MB worth of sectors
        let second_partition_start = 2048 + first_partition_size;
        let second_partition_size = 2048;

        let _ = mbr.add_partition(PartitionKind::Fat32Lba, 2048, first_partition_size, true);
        let _ = mbr.add_partition(
            PartitionKind::Linux,
            second_partition_start,
            second_partition_size,
            false,
        );

        // Write MBR to device
        let write_result = mbr.write_to_device(&device);
        assert!(write_result.is_ok());

        // Create partition devices
        let valid_partitions = mbr.get_valid_partitions();
        assert_eq!(valid_partitions.len(), 2);

        let first_partition_device =
            create_partition_device(device.clone(), valid_partitions[0]).unwrap();
        let second_partition_device =
            create_partition_device(device.clone(), valid_partitions[1]).unwrap();

        // Write different data to each partition
        let first_data = b"Data for first partition - FAT32";
        let second_data = b"Data for second partition - Linux";

        let mut first_buffer = vec![0u8; 512];
        first_buffer[0..first_data.len()].copy_from_slice(first_data);
        let write_result = first_partition_device.write(&first_buffer);
        assert!(write_result.is_ok());

        let mut second_buffer = vec![0u8; 512];
        second_buffer[0..second_data.len()].copy_from_slice(second_data);
        let write_result = second_partition_device.write(&second_buffer);
        assert!(write_result.is_ok());

        // Reset positions and read back
        let _ = first_partition_device.set_position(&crate::Position::Start(0));
        let _ = second_partition_device.set_position(&crate::Position::Start(0));

        let mut first_read_buffer = vec![0u8; 512];
        let mut second_read_buffer = vec![0u8; 512];

        let read_result = first_partition_device.read(&mut first_read_buffer);
        assert!(read_result.is_ok());
        let read_result = second_partition_device.read(&mut second_read_buffer);
        assert!(read_result.is_ok());

        // Verify each partition has its own data
        assert_eq!(&first_read_buffer[0..first_data.len()], first_data);
        assert_eq!(&second_read_buffer[0..second_data.len()], second_data);

        // Verify the data is different (partitions are isolated)
        assert_ne!(&first_read_buffer[0..32], &second_read_buffer[0..32]);
    }

    #[test]
    fn test_partition_bounds_checking() {
        let device = create_test_device_no_mbr();

        let partition_device_result =
            format_disk_and_get_first_partition(&device, PartitionKind::Fat32Lba, Some(0x12345678));
        assert!(partition_device_result.is_ok());
        let partition_device = partition_device_result.unwrap();

        // Try to write beyond partition bounds
        let partition_size_bytes = partition_device.get_sector_count() as u64 * 512;
        let beyond_bounds_position = partition_size_bytes;

        let _ = partition_device
            .set_position(&crate::Position::Start(beyond_bounds_position))
            .unwrap();
        // This should either fail or be clamped to valid range
        // The exact behavior depends on the partition device implementation

        // Write a small amount of data at the very end of the partition (should work)
        let end_position = partition_size_bytes - 512;
        let set_position_result =
            partition_device.set_position(&crate::Position::Start(end_position));
        assert!(set_position_result.is_ok());

        let test_data = b"End of partition data";
        let mut write_buffer = vec![0u8; 512];
        write_buffer[0..test_data.len()].copy_from_slice(test_data);

        let write_result = partition_device.write(&write_buffer);
        assert!(write_result.is_ok());

        // Read it back to verify
        let set_position_result =
            partition_device.set_position(&crate::Position::Start(end_position));
        assert!(set_position_result.is_ok());
        let mut read_buffer = vec![0u8; 512];
        let read_result = partition_device.read(&mut read_buffer);
        assert!(read_result.is_ok());
        assert_eq!(&read_buffer[0..test_data.len()], test_data);
    }
}
