use alloc::{collections::VecDeque, string::String, sync::Arc};

use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use file_system::{Error, Result, Size};

/// A pipe is a FIFO (ring) buffer that can be used to communicate between tasks.
#[derive(Debug, Clone)]
pub struct Pipe(Arc<RwLock<CriticalSectionRawMutex, VecDeque<u8>>>);

impl Pipe {
    /// Create a new pipe with a buffer of the specified size.
    pub fn new(buffer_size: usize) -> Self {
        Pipe(Arc::new(RwLock::new(VecDeque::with_capacity(buffer_size))))
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

        Ok(Size::new(length as u64))
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

        Ok(Size::new(length as u64))
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

        Ok(Size::new(length as u64))
    }
}
