
// - Dependencies
// - - Local dependencies
use super::Drive::*;
use super::{Chip_select_traits, SPI_traits};
use crate::File_system::Generics::Partition::*;
// - - External dependencies
use embedded_sdmmc::{VolumeManager, SdCard};

// - Types
pub struct Partition_type<SPI: SPI_traits, Chip_select: Chip_select_traits>(
    u8,
    VolumeManager<SdCard<SPI, Chip_select, Delayer_type>, Chip_select>,
);

// - Implementations
// - - Local implementations
impl<SPI, Chip_select> Partition_type<a, SPI, Chip_select> {
    pub fn New(Index: u8, Drive: &'a Drive_type<SPI, Chip_select>) -> Partition_type {
        Partition_type(Index, Drive)
    }

    pub fn Get_drive(&self) -> &Drive_type<SPI, Chip_select> {
        &self.1
    }

    pub fn Get_drive_mut(&mut self) -> &mut Drive_type<SPI, Chip_select> {
        &mut self.1
    }

    pub fn Get_index(&self) -> u8 {
        self.0
    }
}

// - - Partition_traits
impl Partition_traits for Partition_type {
    fn Get_type(&self) -> Partition_type {
        todo!();
    }
}
