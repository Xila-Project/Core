#![cfg(target_os = "espidf")]

use embedded_sdmmc::{SdCard, TimeSource};


use crate::File_system::Generics::File_system::File_system_traits;

use super::super::Generics::{Drive::*, Fundamentals::*, Master_boot_record::*};

struct Delayer_type;

impl embedded_hal::blocking::delay::DelayUs<u8> for Delayer_type {
    fn delay_us(&mut self, us: u8) {
        esp_idf_hal::delay::Ets::delay_us(us as u32);
    }
}

pub struct SD_SPI_clock_type;

impl TimeSource for SD_SPI_clock_type {
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

pub struct SD_SPI_drive_type<SPI, Chip_select, Delayer>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8> + embedded_hal::blocking::spi::Write<u8>,
    Chip_select: embedded_hal::digital::v2::OutputPin,
    <SPI as embedded_hal::blocking::spi::Transfer<u8>>::Error: core::fmt::Debug,
    <SPI as embedded_hal::blocking::spi::Write<u8>>::Error: core::fmt::Debug,
    Delayer: embedded_hal::blocking::delay::DelayUs<u8>,
{
    Inner: SdCard<SPI, Chip_select, Delayer>,
}

impl<SPI, Chip_select, Delayer> SD_SPI_drive_type<SPI, Chip_select, Delayer>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8> + embedded_hal::blocking::spi::Write<u8>,
    Chip_select: embedded_hal::digital::v2::OutputPin,
    <SPI as embedded_hal::blocking::spi::Transfer<u8>>::Error: core::fmt::Debug,
    <SPI as embedded_hal::blocking::spi::Write<u8>>::Error: core::fmt::Debug,
    Delayer: embedded_hal::blocking::delay::DelayUs<u8>,
{
    pub fn New(
        SPI: SPI,
        Chip_select: Chip_select,
        Delayer: Delayer,
    ) -> SD_SPI_drive_type<SPI, Chip_select, Delayer> {
        SD_SPI_drive_type {
            Inner: SdCard::new(SPI, Chip_select, Delayer),
        }
    }
}

impl<SPI, Chip_select, Delayer> Drive_traits for SD_SPI_drive_type
where
    SPI: embedded_hal::blocking::spi::Transfer<u8> + embedded_hal::blocking::spi::Write<u8>,
    Chip_select: embedded_hal::digital::v2::OutputPin,
    <SPI as embedded_hal::blocking::spi::Transfer<u8>>::Error: core::fmt::Debug,
    <SPI as embedded_hal::blocking::spi::Write<u8>>::Error: core::fmt::Debug,
    Delayer: embedded_hal::blocking::delay::DelayUs<u8>,
{
    fn Initialize(&self) -> Result<(), ()> {
        self.Inner.init().map_err(|_| ())
    }

    fn Get_Usable_Size(&self) -> Result<Size_type, ()> {
        self.Inner.num_bytes().map_err(|_| ())
    }

    fn Get_Block_Count(&self) -> Result<Size_type, ()> {
        self.Inner.num_blocks().map_err(|_| ())
    }

    fn Read(&self, Start_block_index: Size_type, Buffer: &mut [Block_type]) -> Result<(), ()> {
        self.Inner
            .read_blocks(Start_block_index.0, Buffer)
            .map_err(|_| ())
    }

    fn Write(&self, Start_block_index: Size_type, Blocks: &[Block_type]) -> Result<(), ()> {
        self.Inner
            .write_blocks(Start_block_index.0, Blocks)
            .map_err(|_| ())
    }

    fn Read_master_boot_record(&self) -> Result<Master_boot_record_type, ()> {
        self.Inner
            .read_mbr()
            .map(|mbr| Master_boot_record_type::from(mbr))
            .map_err(|_| ())
    }

    fn Deinitialize(&self) -> Result<(), ()> {
        self.Inner.deinit().map_err(|_| ())
    }
}

pub struct SD_SPI_file_system_type<SPI, Chip_select, Delayer>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8> + embedded_hal::blocking::spi::Write<u8>,
    Chip_select: embedded_hal::digital::v2::OutputPin,
    <SPI as embedded_hal::blocking::spi::Transfer<u8>>::Error: core::fmt::Debug,
    <SPI as embedded_hal::blocking::spi::Write<u8>>::Error: core::fmt::Debug,
    Delayer: embedded_hal::blocking::delay::DelayUs<u8>,
{
    Drive: SD_SPI_drive_type<SPI, Chip_select, Delayer>,
}

impl<SPI, Chip_select, Delayer> File_system_traits for SD_SPI_drive_type<SPI, Chip_select, Delayer>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8> + embedded_hal::blocking::spi::Write<u8>,
    Chip_select: embedded_hal::digital::v2::OutputPin,
    <SPI as embedded_hal::blocking::spi::Transfer<u8>>::Error: core::fmt::Debug,
    <SPI as embedded_hal::blocking::spi::Write<u8>>::Error: core::fmt::Debug,
    Delayer: embedded_hal::blocking::delay::DelayUs<u8>,
{
    fn Initialize(&mut self) -> Result<(), ()> {
        Ok(())
    }

    fn Exists(&self, Path : &str) -> Result<bool, ()> {
        Ok(self.Drive.Inner.exists(Path))
    }


}