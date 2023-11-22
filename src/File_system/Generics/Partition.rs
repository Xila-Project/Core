use super::Fundamentals::*;

pub struct Partition_type_type(pub u8);

pub trait Partition_traits {
    pub fn Get_type(&self) -> Partition_type_type;
}
