use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crate::{Error_type, Result_type, Size_type};

/// A pipe is a FIFO (ring) buffer that can be used to communicate between tasks.
#[derive(Clone, Debug)]
pub struct Pipe_type(Arc<RwLock<VecDeque<u8>>>);

impl Pipe_type {
    /// Create a new pipe with a buffer of the specified size.
    pub fn New(Buffer_size: usize) -> Self {
        Self(Arc::new(RwLock::new(VecDeque::with_capacity(Buffer_size))))
    }

    pub fn Write(&self, Data: &[u8]) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        if Data.len() > Inner.capacity() - Inner.len() {
            return Err(Error_type::Ressource_busy);
        }

        for Byte in Data {
            Inner.push_back(*Byte);
        }

        Ok(())
    }

    pub fn Read(&self, Data: &mut [u8]) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        if Data.len() > Inner.len() {
            return Err(Error_type::Ressource_busy);
        }

        for Byte in Data {
            *Byte = Inner.pop_front().unwrap();
        }

        Ok(())
    }

    pub fn Get_size(&self) -> Result_type<Size_type> {
        Ok(self.0.read()?.len().into())
    }
}
