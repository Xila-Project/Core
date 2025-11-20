use file_system::{DirectBaseOperations, DirectCharacterDevice, MountOperations, Size};
use graphics::InputData;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

pub struct PointerDevice(&'static RwLock<CriticalSectionRawMutex, InputData>);

impl PointerDevice {
    pub fn new(signal: &'static RwLock<CriticalSectionRawMutex, InputData>) -> Self {
        Self(signal)
    }
}

impl DirectBaseOperations for PointerDevice {
    fn read(&self, buffer: &mut [u8], _: Size) -> file_system::Result<usize> {
        // - Cast the pointer data to the buffer.
        let data: &mut InputData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        // Copy the pointer data.

        if let Ok(guard) = self.0.try_read() {
            *data = *guard;
        }

        Ok(size_of::<InputData>())
    }

    fn write(&self, _: &[u8], _: Size) -> file_system::Result<usize> {
        Err(file_system::Error::UnsupportedOperation)
    }
}

impl MountOperations for PointerDevice {}

impl DirectCharacterDevice for PointerDevice {}
