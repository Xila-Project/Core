use std::sync::{Arc, Mutex};

use file_system::{DeviceTrait, Size};
use graphics::{ScreenReadData, ScreenWriteData};

use super::Inner;

pub struct ScreenDevice(Arc<Mutex<Inner>>);

unsafe impl Sync for ScreenDevice {}

unsafe impl Send for ScreenDevice {}

impl ScreenDevice {
    pub fn new(inner: Arc<Mutex<Inner>>) -> Self {
        Self(inner)
    }
}

impl DeviceTrait for ScreenDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        let data: &mut ScreenReadData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        let resolution = self.0.lock().unwrap().get_resolution().unwrap();

        data.set_resolution(resolution);

        Ok(Size::new(size_of::<Self>() as u64))
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<file_system::Size> {
        let data: &ScreenWriteData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        self.0.lock().unwrap().draw(data).unwrap();

        Ok(Size::new(size_of::<Self>() as u64))
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        Ok(Size::new(size_of::<Self>() as u64))
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<file_system::Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
