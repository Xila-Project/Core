use std::sync::{Arc, Mutex};

use file_system::{DeviceTrait, Size};
use graphics::Input_data_type;

use super::Inner;

pub struct PointerDeviceType(Arc<Mutex<Inner>>);

impl PointerDeviceType {
    pub fn new(inner: Arc<Mutex<Inner>>) -> Self {
        Self(inner)
    }
}

impl DeviceTrait for PointerDeviceType {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<Size> {
        // - Cast the pointer data to the buffer.
        let data: &mut Input_data_type = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        // Copy the pointer data.
        *data = *self.0.lock().unwrap().get_pointer_data().unwrap();

        Ok(size_of::<Input_data_type>().into())
    }

    fn write(&self, _: &[u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(size_of::<Input_data_type>().into())
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
