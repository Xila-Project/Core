use esp_idf_hal;

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use embedded_sdmmc::*;



use super::{Drive::*, Fundamentals::*, Partition::*, Master_boot_record::*};

use std::io::{Read, Seek, Write};

// - Custom delayer for the SD card

struct Delayer;

impl DelayUs<u8> for Delayer {
    fn delay_us(&mut self, us: u8) {
        unsafe {
            esp_idf_hal::delay::ets_delay_us(us as u32);
        }
    }
}

pub struct SD_Card_structure<SPI_type, Select_type>
where
    SPI_type: Transfer<u8> + Write<u8>,
    Select_type: OutputPin,
    <SPI_type as Transfer<u8>>::Error: core::fmt::Debug,
    <SPI_type as Write<u8>>::Error: core::fmt::Debug,
{
    SD_card: SdCard<SPI_type, Select_type, Delayer>,
}

impl<SPI_type, Select_type> SD_Card_structure<SPI_type, Select_type>
where
    SPI_type: Transfer<u8> + Write<u8>,
    Select_type: embedded_hal::digital::v2::OutputPin,
    <SPI_type as Transfer<u8>>::Error: core::fmt::Debug,
    <SPI_type as Write<u8>>::Error: core::fmt::Debug,
{
    pub fn New(SPI: SPI_type, Select: Select_type) -> Self {
        SD_Card_structure {
            SD_card: SdCard::new(SPI, Select, Delayer {}),
        }
    }
}

impl Into<BlockIdx> for Size_type {
    fn into(self) -> BlockIdx {
        BlockIdx(self.0 as u32)
    }
}

impl From<BlockIdx> for Size_type {
    fn from(Block_index: BlockIdx) -> Self {
        Size_type(Block_index.0 as u64)
    }
}

impl Into<Block> for Block_type {
    fn into(self) -> Block {
        Block { contents: self.0 }
    }
}

impl From<Block> for Block_type {
    fn from(Block: Block) -> Self {
        Block_type(Block.contents)
    }
}

impl<SPI_type, Select_type> Read for SD_Card_structure<SPI_type, Select_type>
where
    SPI_type: Transfer<u8> + Write<u8>,
    Select_type: embedded_hal::digital::v2::OutputPin,
    <SPI_type as Transfer<u8>>::Error: core::fmt::Debug,
    <SPI_type as Write<u8>>::Error: core::fmt::Debug,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {}
}

impl<SPI_type, Select_type> Drive_traits for SD_Card_structure<SPI_type, Select_type>
where
    SPI_type: Transfer<u8> + Write<u8>,
    Select_type: embedded_hal::digital::v2::OutputPin,
    <SPI_type as Transfer<u8>>::Error: core::fmt::Debug,
    <SPI_type as Write<u8>>::Error: core::fmt::Debug,
{
    fn Initialize(&self) -> Result<(), ()> {
        match self.Get_Usable_Size() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn Deinitialize(&self) -> Result<(), ()> {
        Ok(())
    }

    fn Get_Usable_Size(&self) -> Result<Size_type, ()> {
        match self.SD_card.num_bytes() {
            Ok(size) => Ok(Size_type(size)),
            Err(_) => Err(()),
        }
    }

    fn Get_Block_Count(&self) -> Result<Size_type, ()> {
        match self.SD_card.num_blocks() {
            Ok(size) => Ok(Size_type(size.0 as u64)),
            Err(_) => Err(()),
        }
    }

    fn Read(&self, Start_block_index: Size_type, Buffer: &mut [Block_type]) -> Result<(), ()> {
        if Buffer.len() == 1 {
            let mut Buffer_2 = [Block::default()];
            match self
                .SD_card
                .read(&mut Buffer_2, Start_block_index.into(), "")
            {
                Ok(_) => {
                    Buffer[0] = Buffer_2[0].into();
                    return Ok(());
                }
                Err(_) => return Err(()),
            }
        }

        let mut i = 0;
        while i < Buffer.len() {
            const Blocks_per_read: usize = 5;

            let mut Buffer_2: [Block; Blocks_per_read] = Default::default();

            let Current_block_index = Size_type(Start_block_index.0 + i as u64);

            match self
                .SD_card
                .read(&mut Buffer_2, Current_block_index.into(), "")
            {
                Ok(_) => {
                    for j in 0..4 {
                        Buffer[i + j] = Buffer_2[j].into();
                    }
                }
                Err(_) => return Err(()),
            }
            i += Blocks_per_read;
        }
        return Ok(());
    }

    fn Write(&self, Start_block_index: Size_type, Blocks: &[Block_type]) -> Result<(), ()> {
        if Blocks.len() == 1 {
            let mut Blocks_2 = [Block::default()];
            Blocks_2[0] = Blocks[0].into();
            match self.SD_card.write(&Blocks_2, Start_block_index.into()) {
                Ok(_) => return Ok(()),
                Err(_) => return Err(()),
            }
        }

        let mut i = 0;
        while i < Blocks.len() {
            const Blocks_per_write: usize = 5;

            let mut Blocks_2: [Block; Blocks_per_write] = Default::default();

            let Current_block_index = Size_type(Start_block_index.0 + i as u64);

            for j in 0..4 {
                Blocks_2[j] = Blocks[i + j].into();
            }

            match self.SD_card.write(&Blocks_2, Current_block_index.into()) {
                Ok(_) => {}
                Err(_) => return Err(()),
            }
            i += Blocks_per_write;
        }
        return Ok(());
    }

    fn Read_master_boot_record(&self) -> Result<Master_boot_record_type, ()> {
        let mut First_block = [Block_type::default()];
        self.Read(Size_type(0), &mut First_block).map_err(|_| ())?;
        let First_block = First_block[0];

        Master_boot_record_type::try_from(&First_block.0).map_err(|_| ())
    }
}
