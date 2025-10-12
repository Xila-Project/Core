use std::sync::{Arc, Mutex};

use file_system::{DeviceTrait, Size};
use graphics::InputData;

use super::Inner;

pub struct PointerDevice(Arc<Mutex<Inner>>);

impl PointerDevice {
    pub fn new(inner: Arc<Mutex<Inner>>) -> Self {
        Self(inner)
    }
}

impl DeviceTrait for PointerDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<Size> {
        // - Cast the pointer data to the buffer.
        let data: &mut InputData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        // Copy the pointer data.
        *data = *self.0.lock().unwrap().get_pointer_data().unwrap();

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
