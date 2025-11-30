use alloc::collections::VecDeque;

use exported_file_system::{DirectBaseOperations, MountOperations};
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use task::block_on;

use file_system::{Error, Result, Size};

/// A pipe is a FIFO (ring) buffer that can be used to communicate between tasks.
#[derive(Debug)]
pub struct Pipe(RwLock<CriticalSectionRawMutex, VecDeque<u8>>);

impl Pipe {
    /// Create a new pipe with a buffer of the specified size.
    pub fn new(buffer_size: usize) -> Self {
        Pipe(RwLock::new(VecDeque::with_capacity(buffer_size)))
    }

    pub async fn write(&self, data: &[u8]) -> Result<usize> {
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

    pub async fn read(&self, data: &mut [u8]) -> Result<usize> {
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

    pub async fn read_until(&self, buffer: &mut [u8], delimiter: u8) -> Result<usize> {
        let mut internal_buffer = self.0.write().await;

        let mut bytes_read = 0;

        while bytes_read < buffer.len() {
            if internal_buffer.is_empty() {
                break;
            }

            let byte = internal_buffer.pop_front().unwrap();
            buffer[bytes_read] = byte;
            bytes_read += 1;

            if byte == delimiter {
                break;
            }
        }

        if bytes_read == 0 {
            return Err(Error::RessourceBusy);
        }

        Ok(bytes_read)
    }
}

impl DirectBaseOperations for Pipe {
    fn read(&self, buffer: &mut [u8], _absolute_position: Size) -> Result<usize> {
        block_on(self.read(buffer))
    }

    fn write(&self, buffer: &[u8], _absolute_position: Size) -> Result<usize> {
        block_on(self.write(buffer))
    }
}

impl MountOperations for Pipe {}
