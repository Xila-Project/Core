// - Dependencies
// - - Local dependencies
use super::{Chip_select_traits, Partition::*, SPI_traits};
use crate::File_system::Generics::File_system::*;

// - Implementations
impl<SPI, Chip_select> File_system_type<SPI, Chip_select> {
    pub fn New(Partition: Partition_type<SPI, Chip_select>) -> File_system_type<SPI, Chip_select> {
        File_system_type(Partition)
    }

    pub fn Get_partition(&self) -> &Partition_type<SPI, Chip_select> {
        &self.0
    }

    pub fn Get_partition_mut(&mut self) -> &mut Partition_type<SPI, Chip_select> {
        &mut self.0
    }
}
