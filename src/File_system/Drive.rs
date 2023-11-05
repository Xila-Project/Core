
use super::{Partition::*, Fundamentals::*, Master_boot_record::*};
use std::io::{Read, Write, Seek};

pub trait Drive_traits : Read + Write + Seek {
    fn Initialize(&self) -> Result<(), ()>;
    fn Get_Usable_Size(&self) -> Result<Size_type, ()>;
    fn Get_Block_Count(&self) -> Result<Size_type, ()>;
    fn Read(&self, Start_block_index : Size_type, Buffer : &mut [Block_type]) -> Result<(), ()>;
    fn Write(&self, Start_block_index : Size_type, Blocks : &[Block_type]) -> Result<(), ()>;
    fn Read_master_boot_record(&self) -> Result<Master_boot_record_type, ()>;
    fn Read_from_partition(&self, Partition : &Partition_type, Start_block_index : Size_type, Buffer : &mut [Block_type]) -> Result<(), ()> 
    {
        let Start_block_index = Partition.Get_start_block_index() + Start_block_index;
        self.Read(Start_block_index, Buffer)
    }
    fn Write_to_partition(&self, Partition : &Partition_type, Start_block_index : Size_type, Blocks : &[Block_type]) -> Result<(), ()> 
    {
        let Start_block_index = Partition.Get_start_block_index() + Start_block_index;
        if Start_block_index + Size_type(Blocks.len() as u64) > Partition.Get_start_block_index() + Partition.Get_block_count() {
            return Err(());
        }
        self.Write(Start_block_index, Blocks)
    }
    fn Deinitialize(&self) -> Result<(), ()>;   
}

