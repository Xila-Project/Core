use alloc::{collections::VecDeque, string::String, sync::Arc};

use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use File_system::{Error_type, Result_type, Size_type};

/// A pipe is a FIFO (ring) buffer that can be used to communicate between tasks.
#[derive(Debug, Clone)]
pub struct Pipe_type(Arc<RwLock<CriticalSectionRawMutex, VecDeque<u8>>>);

impl Pipe_type {
    /// Create a new pipe with a buffer of the specified size.
    pub fn New(Buffer_size: usize) -> Self {
        Pipe_type(Arc::new(RwLock::new(VecDeque::with_capacity(Buffer_size))))
    }

    pub async fn Write(&self, Data: &[u8]) -> Result_type<Size_type> {
        let mut Buffer = self.0.write().await;

        let Length = Data.len().min(Buffer.capacity() - Buffer.len());

        if Length == 0 {
            return Err(Error_type::Ressource_busy);
        }

        for Byte in Data {
            Buffer.push_back(*Byte);
        }

        Ok(Size_type::New(Length as u64))
    }

    pub async fn Read(&self, Data: &mut [u8]) -> Result_type<Size_type> {
        let mut Buffer = self.0.write().await;

        let Length = Data.len().min(Buffer.len());

        if Length == 0 {
            return Err(Error_type::Ressource_busy);
        }

        for Byte in Data {
            *Byte = Buffer.pop_front().unwrap();
        }

        Ok(Size_type::New(Length as u64))
    }

    pub async fn Read_line(&self, Data: &mut String) -> Result_type<Size_type> {
        let mut Buffer = self.0.write().await;

        let Length = Data.len().min(Buffer.len());

        if Length == 0 {
            return Err(Error_type::Ressource_busy);
        }

        for _ in 0..Length {
            let Byte = Buffer.pop_front().unwrap();

            if Byte == b'\n' {
                break;
            }

            Data.push(Byte as char);
        }

        Ok(Size_type::New(Length as u64))
    }
}
