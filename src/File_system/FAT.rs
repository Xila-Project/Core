use super::File::*;
use super::Drive::*;
use super::Partition::*;

use fatfs::{FileSystem, FsOptions};

trait File_system<'a, Drive_type>
where
    Drive_type: Drive_traits
{
    type File_type : File_traits;

    fn New(Drive : &'a Drive_type, Partition : &'a Partition_type) -> Self;

    fn Open_file(&self, Path: &str) -> Self::File_type;
}

struct FAT_type<'a, Drive_type>
where
    Drive_type : Drive_traits
{
    Drive : &'a Drive_type,
    Partition : &'a Partition_type,
    File_system : FileSystem<>,
}

impl<'a, Drive_type> File_system<'a, Drive_type> for FAT_type<'a, Drive_type>
where
    Drive_type : Drive_traits
{

    fn New(Drive : &Drive_type, Partition : &Partition_type) -> Self {
        FAT_type {
            Drive,
            Partition,
            File_system : FileSystem::new()
        }
    }

 
}
