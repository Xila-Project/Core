use file_system::{DirectBaseOperations, DirectCharacterDevice, MountOperations, Size};
use graphics::{ScreenReadData, ScreenWriteData};

use crate::window_screen::inner_window::InnerWindow;

pub struct ScreenDevice<'a>(&'a InnerWindow);

unsafe impl<'a> Sync for ScreenDevice<'a> {}

unsafe impl<'a> Send for ScreenDevice<'a> {}

impl<'a> ScreenDevice<'a> {
    pub fn new(inner: &'a InnerWindow) -> Self {
        Self(inner)
    }
}

impl<'a> DirectBaseOperations for ScreenDevice<'a> {
    fn read(&self, buffer: &mut [u8], _: Size) -> file_system::Result<usize> {
        let data: &mut ScreenReadData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        let resolution = futures::block_on(self.0.get_resolution()).unwrap();

        data.set_resolution(resolution);

        Ok(size_of::<Self>() as _)
    }

    fn write(&self, buffer: &[u8], _: Size) -> file_system::Result<usize> {
        let data: &ScreenWriteData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        futures::block_on(self.0.draw(data)).unwrap();

        Ok(size_of::<Self>() as _)
    }
}

impl MountOperations for ScreenDevice<'_> {}

impl DirectCharacterDevice for ScreenDevice<'_> {}
