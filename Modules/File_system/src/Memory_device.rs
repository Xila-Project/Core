use alloc::vec;
use alloc::vec::Vec;
use Futures::block_on;
use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use crate::{Device_trait, Position_type, Size_type};

pub struct Memory_device_type<const Block_size: usize>(
    RwLock<CriticalSectionRawMutex, (Vec<u8>, usize)>,
);

impl<const Block_size: usize> Memory_device_type<Block_size> {
    pub fn New(Size: usize) -> Self {
        assert!(Size % Block_size == 0);

        let Data: Vec<u8> = vec![0; Size];

        Self(RwLock::new((Data, 0)))
    }

    pub fn From_vec(Data: Vec<u8>) -> Self {
        assert!(Data.len() % Block_size == 0);

        Self(RwLock::new((Data, 0)))
    }

    pub fn Get_block_count(&self) -> usize {
        let Inner = block_on(self.0.read());

        Inner.0.len() / Block_size
    }
}

impl<const Block_size: usize> Device_trait for Memory_device_type<Block_size> {
    fn Read(&self, Buffer: &mut [u8]) -> crate::Result_type<Size_type> {
        let mut Inner = self
            .0
            .try_write()
            .map_err(|_| crate::Error_type::Ressource_busy)?;
        let (Data, Position) = &mut *Inner;

        let Read_size = Buffer.len().min(Data.len().saturating_sub(*Position));
        Buffer[..Read_size].copy_from_slice(&Data[*Position..*Position + Read_size]);
        *Position += Read_size;
        Ok(Read_size.into())
    }

    fn Write(&self, Buffer: &[u8]) -> crate::Result_type<Size_type> {
        let mut Inner = block_on(self.0.write());
        let (Data, Position) = &mut *Inner;

        Data[*Position..*Position + Buffer.len()].copy_from_slice(Buffer);
        *Position += Buffer.len();
        Ok(Buffer.len().into())
    }

    fn Get_size(&self) -> crate::Result_type<Size_type> {
        let Inner = block_on(self.0.read());

        Ok(Size_type::New(Inner.0.len() as u64))
    }

    fn Set_position(&self, Position: &Position_type) -> crate::Result_type<Size_type> {
        let mut Inner = block_on(self.0.write());
        let (Data, Device_position) = &mut *Inner;

        match Position {
            Position_type::Start(Position) => *Device_position = *Position as usize,
            Position_type::Current(Position) => {
                *Device_position = (*Device_position as isize + *Position as isize) as usize
            }
            Position_type::End(Position) => {
                *Device_position = (Data.len() as isize - *Position as isize) as usize
            }
        }

        Ok(Size_type::New(*Device_position as u64))
    }

    fn Erase(&self) -> crate::Result_type<()> {
        let mut Inner = block_on(self.0.write());

        let (Data, Position) = &mut *Inner;

        Data[*Position..*Position + Block_size].fill(0);

        Ok(())
    }

    fn Flush(&self) -> crate::Result_type<()> {
        Ok(())
    }

    fn Get_block_size(&self) -> crate::Result_type<usize> {
        Ok(Block_size)
    }

    fn Dump_device(&self) -> crate::Result_type<Vec<u8>> {
        let Inner = block_on(self.0.read());

        Ok(Inner.0.clone())
    }
}
