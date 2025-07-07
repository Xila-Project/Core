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
//! // Create a new MBR
//! let mut mbr = MBR_type::New_with_signature(0x12345678);
//! // Add a FAT32 partition
//! mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 204800, true).unwrap();
//!
//! // Write MBR to the device
//! mbr.Write_to_device(&device).unwrap();
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

use crate::{Partition_device_type, Partition_statistics_type};

mod Utilities;
pub use Utilities::*;

use crate::{Device_type, Error_type, Partition_entry_type, Partition_type_type, Result_type};

#[cfg(test)]
use crate::Memory_device_type;

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
    pub Partitions: [Partition_entry_type; 4],
    /// Boot signature (0x55AA)
    pub Boot_signature: [u8; 2],
}

impl MBR_type {
    /// MBR signature bytes
    pub const SIGNATURE: [u8; 2] = [0x55, 0xAA];

    /// Size of MBR in bytes
    pub const SIZE: usize = 512;

    /// Maximum number of primary partitions in MBR
    pub const MAXIMUM_PARTITIONS_COUNT: usize = 4;

    /// Create a new empty MBR
    pub fn New() -> Self {
        Self {
            Bootstrap_code: [0; 440],
            Disk_signature: [0; 4],
            Reserved: [0; 2],
            Partitions: [Partition_entry_type::New(); 4],
            Boot_signature: Self::SIGNATURE,
        }
    }

    /// Create a new MBR with a specific disk signature
    pub fn New_with_signature(Disk_signature: u32) -> Self {
        let mut mbr = Self::New();
        mbr.Set_disk_signature(Disk_signature);
        mbr
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
            Partitions: [Partition_entry_type::New(); 4],
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
            let offset = 446 + (I * 16);
            let partition_data = &Data[offset..offset + 16];

            Partition.Bootable = partition_data[0];
            Partition.Start_head = partition_data[1];
            Partition.Start_sector = partition_data[2];
            Partition.Start_cylinder = partition_data[3];
            Partition.Partition_type = partition_data[4];
            Partition.End_head = partition_data[5];
            Partition.End_sector = partition_data[6];
            Partition.End_cylinder = partition_data[7];
            Partition.Start_lba = u32::from_le_bytes([
                partition_data[8],
                partition_data[9],
                partition_data[10],
                partition_data[11],
            ]);
            Partition.Size_sectors = u32::from_le_bytes([
                partition_data[12],
                partition_data[13],
                partition_data[14],
                partition_data[15],
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
        let bytes_written = Device.Write(&Buffer)?;

        if bytes_written.As_u64() < Self::SIZE as u64 {
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
    pub fn Get_valid_partitions(&self) -> Vec<&Partition_entry_type> {
        self.Partitions
            .iter()
            .filter(|Partition| Partition.Is_valid())
            .collect()
    }

    /// Get all valid partitions (mutable)
    pub fn Get_valid_partitions_mut(&mut self) -> Vec<&mut Partition_entry_type> {
        self.Partitions
            .iter_mut()
            .filter(|partition| partition.Is_valid())
            .collect()
    }

    /// Get bootable partition (if any)
    pub fn Get_bootable_partition(&self) -> Option<&Partition_entry_type> {
        self.Partitions
            .iter()
            .find(|partition| partition.Is_bootable())
    }

    /// Get bootable partition (mutable, if any)
    pub fn Get_bootable_partition_mut(&mut self) -> Option<&mut Partition_entry_type> {
        self.Partitions
            .iter_mut()
            .find(|partition| partition.Is_bootable())
    }

    /// Set a partition as bootable (clears bootable flag from other partitions)
    pub fn Set_bootable_partition(&mut self, Index: usize) -> Result_type<()> {
        if Index >= Self::MAXIMUM_PARTITIONS_COUNT {
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
            .any(|partition| partition.Get_partition_type() == Partition_type_type::Gpt_protective)
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
        partition_type: crate::Partition_type_type,
        start_lba: u32,
        size_sectors: u32,
        bootable: bool,
    ) -> Result_type<usize> {
        let slot = self
            .Get_free_partition_slot()
            .ok_or(Error_type::File_system_full)?;

        let New_partition = Partition_entry_type::New_with_params(
            bootable,
            partition_type,
            start_lba,
            size_sectors,
        );

        // Check for overlaps with existing partitions
        for Existing in &self.Partitions {
            if Existing.Is_valid() && New_partition.Overlaps_with(Existing) {
                return Err(Error_type::Already_exists);
            }
        }

        self.Partitions[slot] = New_partition;

        // If this is the only bootable partition or no other bootable partition exists
        if bootable {
            self.Set_bootable_partition(slot)?;
        }

        Ok(slot)
    }

    /// Remove a partition by index
    pub fn Remove_partition(&mut self, Index: usize) -> Result_type<()> {
        if Index >= Self::MAXIMUM_PARTITIONS_COUNT {
            return Err(Error_type::Invalid_parameter);
        }

        self.Partitions[Index].Clear();
        Ok(())
    }

    /// Check for partition overlaps
    pub fn Has_overlapping_partitions(&self) -> bool {
        let valid_partitions = self.Get_valid_partitions();

        for (I, Partition1) in valid_partitions.iter().enumerate() {
            for partition2 in valid_partitions.iter().skip(I + 1) {
                if Partition1.Overlaps_with(partition2) {
                    return true;
                }
            }
        }

        false
    }

    /// Get partition count
    pub fn Get_partition_count(&self) -> usize {
        self.Partitions.iter().filter(|p| p.Is_valid()).count()
    }

    /// Create partition devices for all valid partitions in this MBR.
    ///
    /// This method iterates through all partition entries in this MBR and creates
    /// [`Partition_device_type`] instances for each valid partition. This is useful
    /// when you need to access all partitions on a disk programmatically.
    ///
    /// # Arguments
    ///
    /// * `Base_device` - The underlying storage device containing all partitions
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Partition_device_type>)` - Vector of partition devices for all valid partitions
    /// * `Err(Error_type)` - Error if any partition device creation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate alloc;
    /// use File_system::*;
    ///
    /// let device = Create_device!(Memory_device_type::<512>::New(4 * 1024 * 1024));
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
    ///     println!("Partition {}: {} sectors", i, partition.Get_sector_count());
    /// }
    /// ```
    pub fn Create_all_partition_devices(
        &self,
        base_device: Device_type,
    ) -> Result_type<Vec<Partition_device_type>> {
        let mut devices = Vec::new();

        for Partition in &self.Partitions {
            if Partition.Is_valid() {
                let device = Create_partition_device(base_device.clone(), Partition)?;
                devices.push(device);
            }
        }

        Ok(devices)
    }

    /// Find partitions of a specific type within this MBR.
    ///
    /// This method searches through all partitions in this MBR and returns references
    /// to those that match the specified partition type. This is useful for locating
    /// specific types of partitions (e.g., FAT32, Linux, etc.) without creating
    /// partition devices.
    ///
    /// # Arguments
    ///
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
    /// use File_system::*;
    ///
    /// let device = Create_device!(Memory_device_type::<512>::New(4 * 1024 * 1024));
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
    pub fn Find_partitions_by_type(
        &self,
        partition_type: crate::Partition_type_type,
    ) -> Vec<(usize, &Partition_entry_type)> {
        self.Partitions
            .iter()
            .enumerate()
            .filter(|(_, partition)| {
                partition.Is_valid() && partition.Get_partition_type() == partition_type
            })
            .collect()
    }

    /// Validate this MBR structure for consistency and correctness.
    ///
    /// This method performs comprehensive validation of this MBR structure, checking:
    /// - MBR signature validity (0x55AA boot signature)
    /// - Partition overlap detection
    /// - Bootable partition count (at most one partition should be bootable)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - MBR is valid and consistent
    /// * `Err(Error_type::Corrupted)` - MBR is invalid or corrupted
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate alloc;
    /// use File_system::*;
    ///
    /// let device = Create_device!(Memory_device_type::<512>::New(4 * 1024 * 1024));
    /// // First create and write a valid MBR
    /// let mut mbr = MBR_type::New_with_signature(0x12345678);
    /// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
    /// mbr.Write_to_device(&device).unwrap();
    ///
    /// // Read it back and validate
    /// let mbr = MBR_type::Read_from_device(&device).unwrap();
    /// match mbr.Validate() {
    ///     Ok(()) => println!("MBR is valid"),
    ///     Err(Error_type::Corrupted) => println!("MBR is corrupted"),
    ///     Err(e) => println!("Validation error: {}", e),
    /// }
    /// ```
    pub fn Validate(&self) -> Result_type<()> {
        // Check MBR signature
        if !self.Is_valid() {
            return Err(Error_type::Corrupted);
        }

        // Check for overlapping partitions
        if self.Has_overlapping_partitions() {
            return Err(Error_type::Corrupted);
        }

        // Check that only one partition is bootable
        let Bootable_count = self.Partitions.iter().filter(|P| P.Is_bootable()).count();

        if Bootable_count > 1 {
            return Err(Error_type::Corrupted);
        }

        Ok(())
    }

    /// Generate comprehensive statistics from this MBR.
    ///
    /// This method analyzes all partitions in this MBR and generates
    /// detailed statistics about partition types, sizes, and other characteristics.
    ///
    /// # Returns
    ///
    /// A new `Partition_statistics_type` containing the computed statistics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate alloc;
    /// use File_system::*;
    ///
    /// let device = Create_device!(Memory_device_type::<512>::New(4 * 1024 * 1024));
    /// // Create an MBR with some partitions
    /// let mut mbr = MBR_type::New_with_signature(0x12345678);
    /// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
    /// mbr.Add_partition(Partition_type_type::Linux, 4096, 2048, false).unwrap();
    /// mbr.Write_to_device(&device).unwrap();
    ///
    /// // Read it back and analyze
    /// let mbr = MBR_type::Read_from_device(&device).unwrap();
    /// let stats = mbr.Generate_statistics();
    /// if stats.Total_partitions > 0 {
    ///     println!("Average partition size: {} sectors",
    ///              stats.Total_used_sectors / stats.Total_partitions as u64);
    /// }
    /// ```
    pub fn Generate_statistics(&self) -> Partition_statistics_type {
        let valid_partitions: Vec<_> = self.Get_valid_partitions();

        let Total_partitions = valid_partitions.len();
        let bootable_partitions = valid_partitions.iter().filter(|P| P.Is_bootable()).count();

        let Fat_partitions = valid_partitions
            .iter()
            .filter(|P| P.Get_partition_type().Is_fat())
            .count();

        let Linux_partitions = valid_partitions
            .iter()
            .filter(|P| P.Get_partition_type().Is_linux())
            .count();

        let Hidden_partitions = valid_partitions
            .iter()
            .filter(|P| P.Get_partition_type().Is_hidden())
            .count();

        let Extended_partitions = valid_partitions
            .iter()
            .filter(|P| P.Get_partition_type().Is_extended())
            .count();

        let Unknown_partitions = valid_partitions
            .iter()
            .filter(|P| {
                matches!(
                    P.Get_partition_type(),
                    crate::Partition_type_type::Unknown(_)
                )
            })
            .count();

        let Total_used_sectors = valid_partitions
            .iter()
            .map(|p| p.Get_size_sectors() as u64)
            .sum();

        let Largest_partition_sectors = valid_partitions
            .iter()
            .map(|p| p.Get_size_sectors())
            .max()
            .unwrap_or(0);

        let Smallest_partition_sectors = valid_partitions
            .iter()
            .map(|p| p.Get_size_sectors())
            .min()
            .unwrap_or(0);

        Partition_statistics_type {
            Total_partitions,
            Bootable_partitions: bootable_partitions,
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
    /// * `Err(Error_type::Invalid_parameter)` - Disk is too small for a partition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use File_system::*;
    ///
    /// // Create MBR for a 4MB device (8192 sectors)
    /// let mbr = MBR_type::Create_basic(0x12345678, Partition_type_type::Fat32_lba, 8192).unwrap();
    ///
    /// // The MBR will have one FAT32 partition starting at sector 2048
    /// let partitions = mbr.Get_valid_partitions();
    /// assert_eq!(partitions.len(), 1);
    /// assert_eq!(partitions[0].Get_start_lba(), 2048);
    /// ```
    pub fn Create_basic(
        disk_signature: u32,
        partition_type: crate::Partition_type_type,
        total_sectors: u32,
    ) -> Result_type<Self> {
        let mut mbr = Self::New_with_signature(disk_signature);

        // Leave some space at the beginning (typically 2048 sectors for alignment)
        let Start_lba = 2048;
        let partition_sectors = total_sectors.saturating_sub(Start_lba);

        if partition_sectors == 0 {
            return Err(Error_type::Invalid_parameter);
        }

        mbr.Add_partition(partition_type, Start_lba, partition_sectors, true)?;

        Ok(mbr)
    }

    /// Convert MBR back to bytes
    pub fn To_bytes(&self) -> [u8; Self::SIZE] {
        let mut buffer = [0u8; Self::SIZE];

        // Copy bootstrap code
        buffer[0..440].copy_from_slice(&self.Bootstrap_code);

        // Copy disk signature
        buffer[440..444].copy_from_slice(&self.Disk_signature);

        // Copy reserved bytes
        buffer[444..446].copy_from_slice(&self.Reserved);

        // Copy partition entries
        for (I, Partition) in self.Partitions.iter().enumerate() {
            let offset = 446 + (I * 16);
            buffer[offset] = Partition.Bootable;
            buffer[offset + 1] = Partition.Start_head;
            buffer[offset + 2] = Partition.Start_sector;
            buffer[offset + 3] = Partition.Start_cylinder;
            buffer[offset + 4] = Partition.Partition_type;
            buffer[offset + 5] = Partition.End_head;
            buffer[offset + 6] = Partition.End_sector;
            buffer[offset + 7] = Partition.End_cylinder;

            let Start_lba_bytes = Partition.Start_lba.to_le_bytes();
            buffer[offset + 8..offset + 12].copy_from_slice(&Start_lba_bytes);

            let Size_bytes = Partition.Size_sectors.to_le_bytes();
            buffer[offset + 12..offset + 16].copy_from_slice(&Size_bytes);
        }

        // Copy boot signature
        buffer[510..512].copy_from_slice(&self.Boot_signature);

        buffer
    }

    /// Find or create a partition with a specific disk signature.
    ///
    /// This function searches for an MBR with the specified disk signature on the device.
    /// If found, it returns a partition device for the first valid partition. If not found,
    /// or if no valid partitions exist, it formats the disk with a new MBR containing a
    /// single partition that occupies the full disk space.
    ///
    /// # Arguments
    ///
    /// * `Device` - The storage device to search or format
    /// * `Target_signature` - The disk signature to look for
    /// * `Partition_type` - The type of partition to create if formatting is needed
    ///
    /// # Returns
    ///
    /// * `Ok(Partition_device_type)` - Partition device for the found or created partition
    /// * `Err(Error_type)` - Error if operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate alloc;
    /// use File_system::*;
    ///
    /// let device = Create_device!(Memory_device_type::<512>::New(4 * 1024 * 1024));
    ///
    /// // Look for partition with signature 0x12345678, create FAT32 partition if not found
    /// let partition = MBR_type::Find_or_create_partition_with_signature(
    ///     &device,
    ///     0x12345678,
    ///     Partition_type_type::Fat32_lba
    /// ).unwrap();
    ///
    /// // The partition device is ready to use
    /// assert!(partition.Is_valid());
    /// ```
    pub fn Find_or_create_partition_with_signature(
        device: &Device_type,
        target_signature: u32,
        partition_type: crate::Partition_type_type,
    ) -> Result_type<Partition_device_type> {
        // Try to read existing MBR from device
        if let Ok(existing_mbr) = Self::Read_from_device(device) {
            // Check if the MBR has the target signature
            if existing_mbr.Get_disk_signature() == target_signature && existing_mbr.Is_valid() {
                // Get valid partitions
                let valid_partitions = existing_mbr.Get_valid_partitions();

                // If we have at least one valid partition, return it
                if !valid_partitions.is_empty() {
                    return Create_partition_device(device.clone(), valid_partitions[0]);
                }
            }
        }

        // Either no MBR found, wrong signature, or no valid partitions
        // Format the disk with new MBR and create partition
        Self::Format_disk_with_signature_and_partition(device, target_signature, partition_type)
    }

    /// Format a disk with a specific signature and create a single partition.
    ///
    /// This function creates a new MBR with the specified disk signature and a single
    /// partition that occupies most of the available disk space, leaving standard
    /// alignment space at the beginning.
    ///
    /// # Arguments
    ///
    /// * `Device` - The storage device to format
    /// * `Disk_signature` - The disk signature to set in the new MBR
    /// * `Partition_type` - The type of partition to create
    ///
    /// # Returns
    ///
    /// * `Ok(Partition_device_type)` - Partition device for the created partition
    /// * `Err(Error_type)` - Error if operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate alloc;
    /// use File_system::*;
    ///
    /// let device = Create_device!(Memory_device_type::<512>::New(4 * 1024 * 1024));
    ///
    /// // Format disk and create a Linux partition with specific signature
    /// let partition = MBR_type::Format_disk_with_signature_and_partition(
    ///     &device,
    ///     0xABCDEF00,
    ///     Partition_type_type::Linux
    /// ).unwrap();
    ///
    /// // Verify the partition was created correctly
    /// assert!(partition.Is_valid());
    /// assert_eq!(partition.Get_start_lba(), 2048); // Standard alignment
    /// ```
    pub fn Format_disk_with_signature_and_partition(
        device: &Device_type,
        disk_signature: u32,
        partition_type: crate::Partition_type_type,
    ) -> Result_type<Partition_device_type> {
        // Get device size in sectors
        let device_size = device.Get_size()?;
        let block_size = device.Get_block_size()?;
        let total_sectors = (device_size.As_u64() / block_size as u64) as u32;

        // Ensure device is large enough for a meaningful partition
        if total_sectors < 2048 {
            return Err(Error_type::Invalid_parameter);
        }

        // Create new MBR with the specified signature
        let new_mbr = Self::Create_basic(disk_signature, partition_type, total_sectors)?;

        // Write the new MBR to device
        new_mbr.Write_to_device(device)?;

        // Get the first (and only) partition
        let valid_partitions = new_mbr.Get_valid_partitions();
        if valid_partitions.is_empty() {
            return Err(Error_type::Internal_error);
        }

        // Create and return partition device
        Create_partition_device(device.clone(), valid_partitions[0])
    }
}

impl Default for MBR_type {
    fn default() -> Self {
        Self::New()
    }
}

impl fmt::Display for MBR_type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "Master Boot Record:")?;
        writeln!(
            formatter,
            "  Disk Signature: 0x{:08X}",
            self.Get_disk_signature()
        )?;
        writeln!(
            formatter,
            "  Boot Signature: 0x{:02X}{:02X}",
            self.Boot_signature[1], self.Boot_signature[0]
        )?;
        writeln!(formatter, "  Valid: {}", self.Is_valid())?;
        writeln!(
            formatter,
            "  GPT Protective: {}",
            self.Has_gpt_protective_partition()
        )?;
        writeln!(
            formatter,
            "  Partition Count: {}",
            self.Get_partition_count()
        )?;
        writeln!(
            formatter,
            "  Has Overlaps: {}",
            self.Has_overlapping_partitions()
        )?;
        writeln!(formatter, "  Partitions:")?;

        for (I, Partition) in self.Partitions.iter().enumerate() {
            if Partition.Is_valid() {
                writeln!(formatter, "    {}: {}", I + 1, Partition)?;
            } else {
                writeln!(formatter, "    {}: Empty", I + 1)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use crate::Partition_entry_type;
    use alloc::{format, vec};

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
        assert_eq!(
            P1.Get_partition_type(),
            super::Partition_type_type::Fat32_lba
        );
        assert_eq!(P1.Get_start_lba(), 2048);
        assert_eq!(P1.Get_size_sectors(), 204800);

        // Check second partition
        let P2 = &Partitions[1];
        assert!(!P2.Is_bootable());
        assert_eq!(P2.Get_partition_type(), super::Partition_type_type::Linux);
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
            super::Partition_type_type::Fat32_lba
        );
    }

    #[test]
    fn Test_mbr_set_bootable_partition() {
        let mut Mbr = super::MBR_type::New();

        // Add two partitions
        Mbr.Add_partition(super::Partition_type_type::Fat32, 2048, 100000, false)
            .unwrap();
        Mbr.Add_partition(super::Partition_type_type::Linux, 102048, 50000, true)
            .unwrap();

        // Second partition should be bootable
        let Bootable = Mbr.Get_bootable_partition().unwrap();
        assert_eq!(
            Bootable.Get_partition_type(),
            super::Partition_type_type::Linux
        );

        // Set first partition as bootable
        Mbr.Set_bootable_partition(0).unwrap();
        let Bootable = Mbr.Get_bootable_partition().unwrap();
        assert_eq!(
            Bootable.Get_partition_type(),
            super::Partition_type_type::Fat32
        );
    }

    #[test]
    fn Test_mbr_add_partition() {
        let mut Mbr = super::MBR_type::New();

        let Index = Mbr
            .Add_partition(super::Partition_type_type::Fat32_lba, 2048, 204800, true)
            .unwrap();

        assert_eq!(Index, 0);
        assert_eq!(Mbr.Get_partition_count(), 1);

        let Partition = &Mbr.Partitions[Index];
        assert!(Partition.Is_valid());
        assert!(Partition.Is_bootable());
        assert_eq!(
            Partition.Get_partition_type(),
            super::Partition_type_type::Fat32_lba
        );
    }

    #[test]
    fn Test_mbr_add_overlapping_partition() {
        let mut Mbr = super::MBR_type::New();

        // Add first partition
        Mbr.Add_partition(super::Partition_type_type::Fat32, 1000, 2000, false)
            .unwrap();

        // Try to add overlapping partition
        let Result = Mbr.Add_partition(super::Partition_type_type::Linux, 1500, 1000, false);
        assert!(Result.is_err());
        assert_eq!(Result.unwrap_err(), crate::Error_type::Already_exists);
    }

    #[test]
    fn Test_mbr_add_too_many_partitions() {
        let mut Mbr = super::MBR_type::New();

        // Fill all 4 partition slots
        for I in 0..4 {
            let Start = (I as u32) * 10000 + 1000;
            Mbr.Add_partition(super::Partition_type_type::Linux, Start, 5000, false)
                .unwrap();
        }

        // Try to add fifth partition
        let Result = Mbr.Add_partition(super::Partition_type_type::Fat32, 50000, 1000, false);
        assert!(Result.is_err());
        assert_eq!(Result.unwrap_err(), crate::Error_type::File_system_full);
    }

    #[test]
    fn Test_mbr_remove_partition() {
        let mut Mbr = super::MBR_type::New();

        Mbr.Add_partition(super::Partition_type_type::Fat32, 2048, 100000, false)
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
        Mbr.Add_partition(super::Partition_type_type::Fat32, 1000, 1000, false)
            .unwrap();
        Mbr.Add_partition(super::Partition_type_type::Linux, 3000, 1000, false)
            .unwrap();

        // Should have third slot free
        assert_eq!(Mbr.Get_free_partition_slot(), Some(2));

        // Fill remaining slots
        Mbr.Add_partition(super::Partition_type_type::Linux_swap, 5000, 1000, false)
            .unwrap();
        Mbr.Add_partition(super::Partition_type_type::Ntfs_exfat, 7000, 1000, false)
            .unwrap();

        // Should have no free slots
        assert_eq!(Mbr.Get_free_partition_slot(), None);
    }

    #[test]
    fn Test_mbr_has_gpt_protective() {
        let mut Mbr = super::MBR_type::New();
        assert!(!Mbr.Has_gpt_protective_partition());

        Mbr.Add_partition(
            super::Partition_type_type::Gpt_protective,
            1,
            0xFFFFFFFF,
            false,
        )
        .unwrap();
        assert!(Mbr.Has_gpt_protective_partition());
    }

    #[test]
    fn Test_mbr_overlapping_partitions_detection() {
        let mut Mbr = super::MBR_type::New();

        // Add non-overlapping partitions
        Mbr.Add_partition(super::Partition_type_type::Fat32, 1000, 1000, false)
            .unwrap();
        Mbr.Add_partition(super::Partition_type_type::Linux, 3000, 1000, false)
            .unwrap();
        assert!(!Mbr.Has_overlapping_partitions());

        // Manually create overlapping partitions (bypassing validation)
        Mbr.Partitions[2] = Partition_entry_type::New_with_params(
            false,
            super::Partition_type_type::Linux_swap,
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
        assert_eq!(super::MBR_type::MAXIMUM_PARTITIONS_COUNT, 4);
        assert_eq!(super::MBR_type::SIGNATURE, [0x55, 0xAA]);
    }

    #[test]
    fn Test_find_or_create_partition_with_signature_existing() {
        let Mbr_data = Create_sample_mbr_bytes();
        // Create a larger device (4MB) and put the MBR at the beginning
        let mut Data = vec![0u8; 4096 * 1024];
        Data[0..512].copy_from_slice(&Mbr_data);

        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        let Device = crate::Create_device!(Memory_device);

        // The sample MBR has signature 0x12345678
        let Result = MBR_type::Find_or_create_partition_with_signature(
            &Device,
            0x12345678,
            Partition_type_type::Fat32_lba,
        );
        assert!(Result.is_ok());

        let Partition_device = Result.unwrap();
        assert!(Partition_device.Is_valid());
        assert_eq!(Partition_device.Get_start_lba(), 2048); // From sample data
    }

    #[test]
    fn Test_find_or_create_partition_with_signature_wrong_signature() {
        let Mbr_data = Create_sample_mbr_bytes();
        // Create a larger device (4MB) and put the MBR at the beginning
        let mut Data = vec![0u8; 4096 * 1024];
        Data[0..512].copy_from_slice(&Mbr_data);

        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        let Device = crate::Create_device!(Memory_device);

        // Request different signature than what's in the sample MBR
        let Result = MBR_type::Find_or_create_partition_with_signature(
            &Device,
            0xABCDEF00, // Different from 0x12345678 in sample
            Partition_type_type::Linux,
        );

        assert!(Result.is_ok());

        let Partition_device = Result.unwrap();
        assert!(Partition_device.Is_valid());

        // Verify new MBR was created with correct signature
        let New_mbr = MBR_type::Read_from_device(&Device).unwrap();
        assert_eq!(New_mbr.Get_disk_signature(), 0xABCDEF00);

        // Check partition type
        let Valid_partitions = New_mbr.Get_valid_partitions();
        assert_eq!(Valid_partitions.len(), 1);
        assert_eq!(
            Valid_partitions[0].Get_partition_type(),
            Partition_type_type::Linux
        );
    }

    #[test]
    fn Test_find_or_create_partition_with_signature_no_mbr() {
        // Create device with no MBR
        let Data = vec![0u8; 4096 * 1024]; // 8MB device with no MBR
        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        let Device = crate::Create_device!(Memory_device);

        let Result = MBR_type::Find_or_create_partition_with_signature(
            &Device,
            0x11223344,
            Partition_type_type::Fat32_lba,
        );
        assert!(Result.is_ok());

        let Partition_device = Result.unwrap();
        assert!(Partition_device.Is_valid());
        assert_eq!(Partition_device.Get_start_lba(), 2048); // Standard alignment

        // Verify MBR was created with correct signature
        let Mbr = MBR_type::Read_from_device(&Device).unwrap();
        assert_eq!(Mbr.Get_disk_signature(), 0x11223344);
        assert!(Mbr.Is_valid());
    }

    #[test]
    fn Test_find_or_create_partition_with_signature_empty_mbr() {
        // Create device with valid MBR but no partitions
        let mut Data = vec![0u8; 4096 * 1024];
        let Empty_mbr = MBR_type::New_with_signature(0x55667788);
        let Mbr_bytes = Empty_mbr.To_bytes();
        Data[0..512].copy_from_slice(&Mbr_bytes);

        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        let Device = crate::Create_device!(Memory_device);

        let Result = MBR_type::Find_or_create_partition_with_signature(
            &Device,
            0x55667788, // Same signature as existing MBR
            Partition_type_type::Ntfs_exfat,
        );
        assert!(Result.is_ok());

        let Partition_device = Result.unwrap();
        assert!(Partition_device.Is_valid());

        // Verify partition was created with correct type
        let Mbr = MBR_type::Read_from_device(&Device).unwrap();
        assert_eq!(Mbr.Get_disk_signature(), 0x55667788);
        let Valid_partitions = Mbr.Get_valid_partitions();
        assert_eq!(Valid_partitions.len(), 1);
        assert_eq!(
            Valid_partitions[0].Get_partition_type(),
            Partition_type_type::Ntfs_exfat
        );
    }

    #[test]
    fn Test_format_disk_with_signature_and_partition() {
        let Data = vec![0u8; 2048 * 1024]; // 4MB device
        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        let Device = crate::Create_device!(Memory_device);

        let Result = MBR_type::Format_disk_with_signature_and_partition(
            &Device,
            0x99887766,
            Partition_type_type::Linux_swap,
        );
        assert!(Result.is_ok());

        let Partition_device = Result.unwrap();
        assert!(Partition_device.Is_valid());
        assert_eq!(Partition_device.Get_start_lba(), 2048);

        // Verify MBR
        let Mbr = MBR_type::Read_from_device(&Device).unwrap();
        assert_eq!(Mbr.Get_disk_signature(), 0x99887766);
        assert!(Mbr.Is_valid());

        let Valid_partitions = Mbr.Get_valid_partitions();
        assert_eq!(Valid_partitions.len(), 1);
        assert_eq!(
            Valid_partitions[0].Get_partition_type(),
            Partition_type_type::Linux_swap
        );
        assert!(Valid_partitions[0].Is_bootable()); // Should be bootable by default
    }

    #[test]
    fn Test_format_disk_with_signature_and_partition_device_too_small() {
        // Create very small device (less than 2048 sectors)
        let Data = vec![0u8; 1024]; // 2 sectors only
        let Memory_device = Memory_device_type::<512>::From_vec(Data);
        let Device = crate::Create_device!(Memory_device);

        let Result = MBR_type::Format_disk_with_signature_and_partition(
            &Device,
            0x12345678,
            Partition_type_type::Fat32_lba,
        );
        assert!(Result.is_err());
        assert_eq!(Result.unwrap_err(), Error_type::Invalid_parameter);
    }
}
