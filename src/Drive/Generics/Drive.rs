// - Dependencies
// - - Local dependencies
use super::{Fundamentals::*, Master_boot_record::*};

// - Traits
pub trait Drive_traits {
    type Partition_type;

    fn Get_usable_size(&self) -> Result<Size_type, ()>;
    fn Read_master_boot_record(&self) -> Result<Master_boot_record_type, ()>;
    fn Get_partition(&self, Index : u8) -> Result<Self::Partition_type, ()>;
}