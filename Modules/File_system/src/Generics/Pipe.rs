use std::sync::{Arc, RwLock};

struct Pipe_internal_type<const Buffer_size: usize> {
    Buffer: [u8; Buffer_size],
    Read_index: usize,
    Write_index: usize,
}

pub struct Pipe_type<const Buffer_size: usize>(Arc<RwLock<Pipe_internal_type<Buffer_size>>>);

impl<const Buffer_size: usize> Pipe_type<Buffer_size> {
    pub fn New() -> Self {
        Self(Arc::new(RwLock::new(Pipe_internal_type {
            Buffer: [0; Buffer_size],
            Read_index: 0,
            Write_index: 0,
        })))
    }

    pub fn Write(&self, Data: &[u8]) -> Result<(), ()> {
        if Data.len() > Buffer_size {
            return Err(());
        }

        let mut Pipe = self.0.write().unwrap();

        for Byte in Data {
            // ? : Probably not the most efficient way to do this.
            let Write_index = Pipe.Write_index; // * Make the borrow checker happy.
            Pipe.Buffer[Write_index] = *Byte;
            Pipe.Write_index += 1;

            if Pipe.Write_index == Buffer_size {
                Pipe.Write_index = 0;
            }
        }

        Ok(())
    }

    pub fn Read(&self, Data: &mut [u8]) -> Result<(), ()> {
        if Data.len() > Buffer_size {
            return Err(());
        }

        let mut Pipe = self.0.write().unwrap();

        for Byte in Data {
            *Byte = Pipe.Buffer[Pipe.Read_index];
            Pipe.Read_index += 1;

            if Pipe.Read_index == Buffer_size {
                Pipe.Read_index = 0;
            }
        }

        Ok(())
    }
}
