use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use file_system::{DeviceTrait, Size};
use graphics::InputData;

pub struct PointerDevice(&'static RwLock<CriticalSectionRawMutex, InputData>);

impl PointerDevice {
    pub fn new(signal: &'static RwLock<CriticalSectionRawMutex, InputData>) -> Self {
        Self(signal)
    }
}

impl DeviceTrait for PointerDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<Size> {
        // - Cast the pointer data to the buffer.
        let data: &mut InputData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        // Copy the pointer data.

        if let Ok(guard) = self.0.try_read() {
            *data = *guard;
        }

        Ok(size_of::<InputData>().into())
    }

    fn write(&self, _: &[u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(size_of::<InputData>().into())
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
