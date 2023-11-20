use super::{Fundamentals::*, Partition::*};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub struct Master_boot_record_type {
    Partitions: Vec<Partition_type>,
}

impl Master_boot_record_type {
    pub fn Get_partitions(&self) -> Vec<Partition_type> {
        self.Partitions.clone()
    }

    pub fn Get_partition_count(&self) -> usize {
        self.Partitions.len()
    }

    pub fn Get_partition(&self, Index: usize) -> Option<Partition_type> {
        if Index >= self.Partitions.len() {
            return None;
        }
        Some(self.Partitions[Index])
    }
}

const Partition_table_start: usize = 446;
const Partition_entry_size: usize = 16;
const Footer_start: usize = 510;
const Footer_value: u16 = 0xAA55; // MBR magic number
const Partition_entry_status_index: usize = 0;
const Partition_entry_type_index: usize = 4;
const Partition_entry_lba_start_index: usize = 8;
const Partition_entry_block_count_index: usize = 12;

impl TryFrom<&[u8; 512]> for Master_boot_record_type {
    type Error = &'static str;

    fn try_from(First_block: &[u8; 512]) -> Result<Self, Self::Error> {
        if First_block.len() != 512 {
            return Err("Invalid block size");
        }

        // - Check the MBR signature
        if LittleEndian::read_u16(&First_block[Footer_start..Footer_start + 2]) != Footer_value {
            return Err("Invalid MBR signature");
        }

        // - Read the partition table
        let mut Partitions: Vec<Partition_type> = Vec::new();

        for i in 0..4 {
            let Partition_start = Partition_table_start + (i * Partition_entry_size);

            let Partition_informations =
                &First_block[Partition_start..(Partition_start + Partition_entry_size)];

            // Only 0x80 (8th bit) and 0x00 are valid (bootable, and non-bootable)
            if (Partition_informations[Partition_entry_status_index] & 0x7F) != 0x00 {
                continue;
            }

            let Start_block_index = LittleEndian::read_u32(
                &Partition_informations
                    [Partition_entry_lba_start_index..(Partition_entry_lba_start_index + 4)],
            );

            let Block_count = LittleEndian::read_u32(
                &Partition_informations
                    [Partition_entry_block_count_index..(Partition_entry_block_count_index + 4)],
            );

            let Partition_type = Partition_informations[Partition_entry_type_index];

            Partitions.push(Partition_type::New(
                Size_type(Start_block_index as u64),
                Size_type(Block_count as u64),
                Partition_type_type(Partition_type),
            ));
        }

        Ok(Master_boot_record_type { Partitions })
    }
}

// - Test
#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::io::Cursor;

    #[test]
    fn New() {
        let Partitions = vec![Partition_type::New(
            Size_type(0),
            Size_type(0),
            Partition_type_type(0),
        )];
        let mbr = Master_boot_record_type {
            Partitions: Partitions.clone(),
        };
        assert_eq!(mbr.Get_partition_count(), 1);
        assert_eq!(mbr.Get_partition(0).unwrap(), Partitions[0]);
    }

    fn Generate_MBR_block(Signature: u16, Partitions: Vec<(u8, u32, u32)>, Block: &mut [u8]) {
        if Partitions.len() > 4 {
            panic!("Invalid partition count");
        }

        for i in 0..4 {
            let Partition_entry_start = Partition_table_start + (i * Partition_entry_size);

            let mut Cursor = Cursor::new(&mut Block[Partition_entry_start..Partition_entry_start + Partition_entry_size]);
            if i < Partitions.len() {
                Cursor.set_position(Partition_entry_type_index as u64);
                Cursor.write_u8(Partitions[i].0).unwrap();
                Cursor.set_position(Partition_entry_lba_start_index as u64);
                Cursor.write_u32::<LittleEndian>(Partitions[i].1).unwrap();
                Cursor.set_position(Partition_entry_block_count_index as u64);
                Cursor.write_u32::<LittleEndian>(Partitions[i].2).unwrap();
            } else {
                Cursor.set_position(Partition_entry_status_index as u64);
                Cursor.write_u8(0xFF).unwrap();
            }
        }

        let mut Cursor = Cursor::new(&mut Block[Footer_start..]);
        Cursor.write_u16::<LittleEndian>(Signature).unwrap();
    }

    #[test]
    fn Try_from_valid() {
        for i in 1..5 {
            let mut Partitions = Vec::new();
            for j in 0..i {
                Partitions.push((0x80, i as u32, j as u32));
            }
            assert_eq!(Partitions.len(), i);

            let mut Block: [u8; 512] = [0u8; 512];
            Generate_MBR_block(Footer_value, Partitions.clone(), &mut Block);
            let Master_boot_record = Master_boot_record_type::try_from(&Block);
            assert!(Master_boot_record.is_ok());
            let Master_boot_record = Master_boot_record.unwrap();
            assert_eq!(Master_boot_record.Get_partition_count(), i);
            for j in 0..i {
                let Partition = Master_boot_record.Get_partition(j).unwrap();
                assert_eq!(Partition.Get_type(), Partition_type_type(0x80));
                assert_eq!(Partition.Get_start_block_index(), Size_type(i as u64));
                assert_eq!(Partition.Get_block_count(), Size_type(j as u64));
            }
        }
    }

    #[test]
    fn Try_from_invalid_footer() {
        let mut Block: [u8; 512] = [0u8; 512];
        Generate_MBR_block(0x0000, vec![], &mut Block);
        assert_eq!(
            Master_boot_record_type::try_from(&Block).unwrap_err(),
            "Invalid MBR signature"
        );
        Generate_MBR_block(0x55AA, vec![], &mut Block);
        assert_eq!(
            Master_boot_record_type::try_from(&Block).unwrap_err(),
            "Invalid MBR signature"
        );
        Generate_MBR_block(0x55AB, vec![], &mut Block);
        assert_eq!(
            Master_boot_record_type::try_from(&Block).unwrap_err(),
            "Invalid MBR signature"
        );
        Generate_MBR_block(0x55BA, vec![], &mut Block);
        assert_eq!(
            Master_boot_record_type::try_from(&Block).unwrap_err(),
            "Invalid MBR signature"
        );
        Generate_MBR_block(0x5A55, vec![], &mut Block);
        assert_eq!(
            Master_boot_record_type::try_from(&Block).unwrap_err(),
            "Invalid MBR signature"
        );
    }
}
