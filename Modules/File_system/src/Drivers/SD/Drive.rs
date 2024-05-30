// - Dependencies
// - - Local dependencies
use super::{Chip_select_traits, Partition::Partition_traits, SPI_traits};
use crate::File_system::Generics::{Drive::*, Fundamentals::*, Master_boot_record::*};
// - - External dependencies
use embedded_sdmmc::{BlockDevice, SdCard, TimeSource, VolumeManager};

// - Types and implementations
// - - Delayer
pub struct Delayer_type;

impl embedded_hal::blocking::delay::DelayUs<u8> for Delayer_type {
    fn delay_us(&mut self, us: u8) {
        esp_idf_hal::delay::Ets::delay_us(us as u32);
    }
}

// - - Clock source
pub struct Clock_type;

impl TimeSource for Clock_type {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

// - - Drive
pub struct Drive_type<SPI_type: SPI_traits, Chip_select_type: Chip_select_traits>(
    SdCard<SPI_type, Chip_select_type, Delayer_type>,
    VolumeManager<SdCard<SPI_type, Chip_select_type, Delayer_type>, Chip_select_type>,
);

impl<SPI_type, Chip_select_type> Drive_type<SPI_type, Chip_select_type> {
    pub fn New(
        SPI: SPI_type,
        Chip_select: Chip_select_type,
    ) -> Drive_type<SPI_type, Chip_select_type> {
        let mut Card = SdCard::new(SPI, Chip_select, Delayer_type());
        let mut Volume_manager = VolumeManager::new(Card, Clock_type {});

        Drive_type(Card, Volume_manager)
    }
}

impl<SPI, Chip_select> Drive_traits for Drive_type<SPI, Chip_select> {
    type Partition_type = Partition::Partition_type;

    fn Get_usable_size(&self) -> Result<Size_type, ()> {
        self.0.num_bytes().map_err(|_| ())
    }

    fn Read_master_boot_record(&self) -> Result<Master_boot_record_type, ()> {
        todo!();
        //let mut buffer = [0u8; 512];
        //Master_boot_record_type::New(buffer)
    }

    fn Get_partition(&self, Index: u8) -> Result<Self::Partition_type, ()> {
        Partition_type::New(Index, self)
    }
}
