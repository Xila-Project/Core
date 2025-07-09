use crate::MBR_type;

/// Comprehensive statistics about partitions in an MBR.
///
/// This structure provides detailed statistical information about the partitions
/// present in an MBR, including counts by type, size information, and bootability status.
/// It's useful for disk analysis, partition management tools, and system diagnostics.
///
/// # Fields
///
/// ## Partition Counts
/// * `Total_partitions` - Total number of valid partitions
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
/// let device = Create_device!(Memory_device_type::<512>::New(4 * 1024 * 1024));
/// // Create an MBR with some partitions
/// let mut mbr = MBR_type::New_with_signature(0x12345678);
/// mbr.Add_partition(Partition_type_type::Fat32_lba, 2048, 1024, true).unwrap();
/// mbr.Add_partition(Partition_type_type::Linux, 4096, 2048, false).unwrap();
/// mbr.Write_to_device(&device).unwrap();
///
/// // Read it back and get statistics
/// let mbr = MBR_type::Read_from_device(&device).unwrap();
/// let stats = Partition_statistics_type::From_mbr(&mbr);
/// println!("Total partitions: {}", stats.Total_partitions);
/// println!("Bootable partitions: {}", stats.Bootable_partitions);
/// println!("Total used sectors: {}", stats.Total_used_sectors);
/// ```
#[derive(Debug, Clone)]
pub struct Partition_statistics_type {
    /// Total number of valid partitions in the MBR.
    pub Total_partitions: usize,
    /// Number of partitions marked as bootable.
    pub Bootable_partitions: usize,
    /// Number of FAT file system partitions.
    pub Fat_partitions: usize,
    /// Number of Linux-type partitions.
    pub Linux_partitions: usize,
    /// Number of hidden partitions.
    pub Hidden_partitions: usize,
    /// Number of extended partitions.
    pub Extended_partitions: usize,
    /// Number of partitions with unknown types.
    pub Unknown_partitions: usize,
    /// Total sectors used by all partitions.
    pub Total_used_sectors: u64,
    /// Size of the largest partition in sectors.
    pub Largest_partition_sectors: u32,
    /// Size of the smallest partition in sectors.
    pub Smallest_partition_sectors: u32,
}

impl Partition_statistics_type {
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
    /// A new `Partition_statistics_type` containing the computed statistics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate alloc;
    /// use file_system::*;
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
    /// let stats = Partition_statistics_type::From_mbr(&mbr);
    /// if stats.Total_partitions > 0 {
    ///     println!("Average partition size: {} sectors",
    ///              stats.Total_used_sectors / stats.Total_partitions as u64);
    /// }
    /// ```
    pub fn from_mbr(Mbr: &MBR_type) -> Self {
        Mbr.generate_statistics()
    }
}
