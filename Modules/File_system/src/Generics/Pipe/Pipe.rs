use std::sync::{Arc, RwLock};

use Shared::Ring_buffer_type;

use crate::Prelude::{Error_type, Result, Size_type};

/// A pipe is a FIFO (ring) buffer that can be used to communicate between tasks.
#[derive(Clone)]
pub struct Pipe_type(Arc<RwLock<Ring_buffer_type<u8>>>);

impl Pipe_type {
    /// Create a new pipe with a buffer of the specified size.
    pub fn New(Buffer_size: usize) -> Self {
        Self(Arc::new(RwLock::new(Ring_buffer_type::New(Buffer_size))))
    }

    pub fn Write(&self, Data: &[u8]) -> Result<()> {
        let mut Inner = self.0.write()?;

        if Data.len() > Inner.Get_free_space() {
            return Err(Error_type::File_system_full);
        }

        for Byte in Data {
            if !Inner.Push(*Byte) {
                return Err(Error_type::File_system_full);
            }
        }
        Ok(())
    }

    pub fn Read(&self, Data: &mut [u8]) -> Result<()> {
        let mut Inner = self.0.write()?;
        let Length = Data.len();

        if Length > Inner.Get_used_space() {
            return Err(Error_type::File_system_full);
        }

        for Byte in Data {
            *Byte = Inner.Pop().unwrap();
        }
        Ok(())
    }

    pub fn Get_size(&self) -> Result<Size_type> {
        Ok(self.0.read()?.Get_capacity().into())
    }
}
