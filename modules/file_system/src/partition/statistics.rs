use crate::Mbr;

/// Comprehensive statistics about partitions in an MBR.
///
/// This structure provides detailed statistical information about the partitions
/// present in an MBR, including counts by type, size information, and bootability status.
/// It's useful for disk analysis, partition management tools, and system diagnostics.
///
/// # Fields
///
/// ## Partition Counts
/// * `total_partitions` - Total number of valid partitions
/// * `Bootable_partitions` - Number of partitions marked as bootable
/// * `Fat_partitions` - Number of FAT file system partitions (FAT16, FAT32, etc.)
/// * `Linux_partitions` - Number of Linux-type partitions
/// * `Hidden_partitions` - Number of hidden partitions
/// * `Extended_partitions` - Number of extended partitions
/// * `Unknown_partitions` - Number of partitions with unknown/unrecognized types
///
/// ## Size Information
/// * `Total_used_sectors` - Total sectors used by all partitions
/// * `Largest_partition_sectors` - Size of the largest partition in sectors
/// * `Smallest_partition_sectors` - Size of the smallest partition in sectors
///
/// # Examples
///
/// ```rust
/// extern crate alloc;
/// use file_system::*;
///
/// let device = create_device!(MemoryDevice::<512>::new(4 * 1024 * 1024));
/// // Create an MBR with some partitions
/// let mut mbr = Mbr::new_with_signature(0x12345678);
/// mbr.add_partition(PartitionKind::Fat32Lba, 2048, 1024, true).unwrap();
/// mbr.add_partition(PartitionKind::Linux, 4096, 2048, false).unwrap();
/// mbr.write_to_device(&device).unwrap();
///
/// // Read it back and get statistics
/// let mbr = Mbr::read_from_device(&device).unwrap();
/// let stats = PartitionStatistics::from_mbr(&mbr);
/// println!("Total partitions: {}", stats.total_partitions);
/// println!("Bootable partitions: {}", stats.bootable_partitions);
/// println!("Total used sectors: {}", stats.total_used_sectors);
/// ```
#[derive(Debug, Clone)]
pub struct PartitionStatistics {
    /// Total number of valid partitions in the MBR.
    pub total_partitions: usize,
    /// Number of partitions marked as bootable.
    pub bootable_partitions: usize,
    /// Number of FAT file system partitions.
    pub fat_partitions: usize,
    /// Number of Linux-type partitions.
    pub linux_partitions: usize,
    /// Number of hidden partitions.
    pub hidden_partitions: usize,
    /// Number of extended partitions.
    pub extended_partitions: usize,
    /// Number of partitions with unknown types.
    pub unknown_partitions: usize,
    /// Total sectors used by all partitions.
    pub total_used_sectors: u64,
    /// Size of the largest partition in sectors.
    pub largest_partition_sectors: u32,
    /// Size of the smallest partition in sectors.
    pub smallest_partition_sectors: u32,
}

impl PartitionStatistics {
    /// Generate comprehensive statistics from an MBR.
    ///
    /// This method analyzes all partitions in the provided MBR and generates
    /// detailed statistics about partition types, sizes, and other characteristics.
    ///
    /// # Arguments
    ///
    /// * `Mbr` - The MBR structure to analyze
    ///
    /// # Returns
    ///
    /// A new `PartitionStatistics` containing the computed statistics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate alloc;
    /// use file_system::*;
    ///
    /// let device = create_device!(MemoryDevice::<512>::new(4 * 1024 * 1024));
    /// // Create an MBR with some partitions
    /// let mut mbr = Mbr::new_with_signature(0x12345678);
    /// mbr.add_partition(PartitionKind::Fat32Lba, 2048, 1024, true).unwrap();
    /// mbr.add_partition(PartitionKind::Linux, 4096, 2048, false).unwrap();
    /// mbr.write_to_device(&device).unwrap();
    ///
    /// // Read it back and analyze
    /// let mbr = Mbr::read_from_device(&device).unwrap();
    /// let stats = PartitionStatistics::from_mbr(&mbr);
    /// if stats.total_partitions > 0 {
    ///     println!("Average partition size: {} sectors",
    ///              stats.total_used_sectors / stats.total_partitions as u64);
    /// }
    /// ```
    pub fn from_mbr(mbr: &Mbr) -> Self {
        mbr.generate_statistics()
    }
}
