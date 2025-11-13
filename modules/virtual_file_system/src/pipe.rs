use alloc::{collections::VecDeque, string::String, sync::Arc};

use exported_file_system::DirectFileOperations;
use futures::block_on;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use file_system::{Error, Result, Size};

/// A pipe is a FIFO (ring) buffer that can be used to communicate between tasks.
#[derive(Debug)]
pub struct Pipe(RwLock<CriticalSectionRawMutex, VecDeque<u8>>);

impl Pipe {
    /// Create a new pipe with a buffer of the specified size.
    pub fn new(buffer_size: usize) -> Self {
        Pipe(RwLock::new(VecDeque::with_capacity(buffer_size)))
    }

    pub async fn write(&self, data: &[u8]) -> Result<Size> {
        let mut buffer = self.0.write().await;

        let length = data.len().min(buffer.capacity() - buffer.len());

        if length == 0 {
            return Err(Error::RessourceBusy);
        }

        for byte in data {
            buffer.push_back(*byte);
        }

        Ok(length as _)
    }

    pub async fn read(&self, data: &mut [u8]) -> Result<Size> {
        let mut buffer = self.0.write().await;

        let length = data.len().min(buffer.len());

        if length == 0 {
            return Err(Error::RessourceBusy);
        }

        for byte in data {
            *byte = buffer.pop_front().unwrap();
        }

        Ok(length as _)
    }

    pub async fn read_line(&self, data: &mut String) -> Result<Size> {
        let mut buffer = self.0.write().await;

        let length = data.len().min(buffer.len());

        if length == 0 {
            return Err(Error::RessourceBusy);
        }

        for _ in 0..length {
            let byte = buffer.pop_front().unwrap();

            if byte == b'\n' {
                break;
            }

            data.push(byte as char);
        }

        Ok(length as _)
    }
}

impl DirectFileOperations for Pipe {
    fn read(&self, buffer: &mut [u8], _absolute_position: Size) -> Result<Size> {
        block_on(self.read(buffer))
    }

    fn write(&self, buffer: &[u8], _absolute_position: Size) -> Result<Size> {
        block_on(self.write(buffer))
    }
}
