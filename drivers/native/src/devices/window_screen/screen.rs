use file_system::{DirectBaseOperations, DirectCharacterDevice, MountOperations, Size};
use graphics::{Area, GET_RESOLUTION, RenderingColor, SET_DRAWING_AREA, WAS_RESIZED};
use shared::align_slice_to;

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
    fn read(&self, _: &mut [u8], _: Size) -> file_system::Result<usize> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn write(&self, buffer: &[u8], _: Size) -> file_system::Result<usize> {
        let data: &[RenderingColor] =
            align_slice_to(buffer).ok_or(file_system::Error::InvalidParameter)?;

        futures::block_on(self.0.draw(data)).unwrap();

        Ok(size_of::<Self>() as _)
    }

    fn control(
        &self,
        command: file_system::ControlCommand,
        argument: &mut file_system::ControlArgument,
    ) -> file_system::Result<()> {
        match command {
            SET_DRAWING_AREA => {
                let area: &Area = argument
                    .cast()
                    .ok_or(file_system::Error::InvalidParameter)?;

                futures::block_on(self.0.set_drawing_area(*area)).unwrap();
            }
            GET_RESOLUTION => {
                let resolution: &mut graphics::Point = argument
                    .cast()
                    .ok_or(file_system::Error::InvalidParameter)?;

                *resolution = futures::block_on(self.0.get_resolution()).unwrap();
            }
            WAS_RESIZED => {
                let was_resized: &mut bool = argument
                    .cast()
                    .ok_or(file_system::Error::InvalidParameter)?;

                *was_resized = futures::block_on(self.0.was_resized()).unwrap();
            }
            _ => return Err(file_system::Error::UnsupportedOperation),
        }

        Ok(())
    }
}

impl MountOperations for ScreenDevice<'_> {}

impl DirectCharacterDevice for ScreenDevice<'_> {}
