use alloc::{collections::VecDeque, string::String, sync::Arc};

use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use file_system::{Error_type, Result_type, Size_type};

/// A pipe is a FIFO (ring) buffer that can be used to communicate between tasks.
#[derive(Debug, Clone)]
pub struct Pipe_type(Arc<RwLock<CriticalSectionRawMutex, VecDeque<u8>>>);

impl Pipe_type {
    /// Create a new pipe with a buffer of the specified size.
    pub fn new(buffer_size: usize) -> Self {
        Pipe_type(Arc::new(RwLock::new(VecDeque::with_capacity(buffer_size))))
    }

    pub async fn write(&self, data: &[u8]) -> Result_type<Size_type> {
        let mut buffer = self.0.write().await;

        let length = data.len().min(buffer.capacity() - buffer.len());

        if length == 0 {
            return Err(Error_type::Ressource_busy);
        }

        for byte in data {
            buffer.push_back(*byte);
        }

        Ok(Size_type::new(length as u64))
    }

    pub async fn read(&self, data: &mut [u8]) -> Result_type<Size_type> {
        let mut buffer = self.0.write().await;

        let length = data.len().min(buffer.len());

        if length == 0 {
            return Err(Error_type::Ressource_busy);
        }

        for byte in data {
            *byte = buffer.pop_front().unwrap();
        }

        Ok(Size_type::new(length as u64))
    }

    pub async fn read_line(&self, data: &mut String) -> Result_type<Size_type> {
        let mut buffer = self.0.write().await;

        let length = data.len().min(buffer.len());

        if length == 0 {
            return Err(Error_type::Ressource_busy);
        }

        for _ in 0..length {
            let byte = buffer.pop_front().unwrap();

            if byte == b'\n' {
                break;
            }

            data.push(byte as char);
        }

        Ok(Size_type::new(length as u64))
    }
}
