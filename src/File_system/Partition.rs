use super::Fundamentals::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Partition_type_type(pub u8);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Partition_type {
    Start_block_index: Size_type,
    Block_count: Size_type,
    Type: Partition_type_type,
}

impl Partition_type {
    pub fn New(
        Start_block_index: Size_type,
        Block_count: Size_type,
        Partition_type: Partition_type_type,
    ) -> Self {
        Partition_type {
            Start_block_index,
            Block_count,
            Type: Partition_type,
        }
    }

    pub fn Get_start_block_index(&self) -> Size_type {
        self.Start_block_index
    }

    pub fn Get_block_count(&self) -> Size_type {
        self.Block_count
    }

    pub fn Get_type(&self) -> Partition_type_type {
        self.Type
    }
}
