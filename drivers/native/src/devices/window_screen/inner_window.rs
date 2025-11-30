use alloc::sync::Arc;

use graphics::{Area, ColorRGBA8888, Point, RenderingColor};
use pixels::Pixels;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use winit::window;

struct Inner {
    window: Arc<window::Window>,
    pixels: Pixels<'static>,
    drawing_area: Area,
    was_resized: bool,
}

pub struct InnerWindow(RwLock<CriticalSectionRawMutex, Option<Inner>>);

impl Default for InnerWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl InnerWindow {
    pub const fn new() -> Self {
        Self(RwLock::new(None))
    }

    pub async fn was_resized(&self) -> Result<bool, String> {
        let mut inner = self.0.write().await;

        let inner = inner
            .as_mut()
            .ok_or_else(|| "Window is None.".to_string())?;

        let was_resized = inner.was_resized;

        inner.was_resized = false;

        Ok(was_resized)
    }

    pub async fn resize(&self, new_size: Point) -> Result<(), String> {
        let mut inner = self.0.write().await;

        let inner = inner
            .as_mut()
            .ok_or_else(|| "Window is None.".to_string())?;

        inner
            .pixels
            .resize_buffer(new_size.get_x() as u32, new_size.get_y() as u32)
            .map_err(|error| format!("Pixels buffer resize error: {error:?}"))?;

        inner
            .pixels
            .resize_surface(new_size.get_x() as u32, new_size.get_y() as u32)
            .map_err(|error| format!("Pixels resize error: {error:?}"))?;

        inner.was_resized = true;

        Ok(())
    }

    pub async fn replace(&self, window: Arc<window::Window>, pixels: Pixels<'static>) {
        let new_inner = Inner {
            window,
            pixels,
            drawing_area: Area::default(),
            was_resized: true,
        };

        self.0.write().await.replace(new_inner);
    }

    pub async fn get_resolution(&self) -> Option<Point> {
        let inner = self.0.read().await;

        inner.as_ref().map(|inner| {
            let size = inner.window.inner_size();
            graphics::Point::new(size.width as i16, size.height as i16)
        })
    }

    pub async fn render(&self) -> Result<(), String> {
        let mut inner = self.0.write().await;

        let Inner {
            pixels,
            window,
            drawing_area: _,
            was_resized: _,
        } = inner
            .as_mut()
            .ok_or_else(|| "Window is None.".to_string())?;

        pixels
            .render()
            .map_err(|error| format!("Pixels render error: {error:?}"))?;

        window.request_redraw();

        Ok(())
    }

    pub async fn set_drawing_area(&self, area: Area) -> Result<(), String> {
        let mut inner = self.0.write().await;

        let inner = inner
            .as_mut()
            .ok_or_else(|| "Window is None.".to_string())?;

        inner.drawing_area = area;

        Ok(())
    }

    pub async fn draw(&self, data_buffer: &[RenderingColor]) -> Result<(), String> {
        let mut inner = self.0.write().await;

        let Inner {
            pixels,
            window,
            drawing_area,
            was_resized: _,
        } = inner
            .as_mut()
            .ok_or_else(|| "Window is None.".to_string())?;

        let (frame_width, _) = {
            let size = window.inner_size();
            (size.width as usize, size.height as usize)
        };

        let point_1 = drawing_area.get_point_1();
        let point_2 = drawing_area.get_point_2();

        let frame = pixels.frame_mut();
        let frame = unsafe {
            core::slice::from_raw_parts_mut(
                frame.as_mut_ptr() as *mut ColorRGBA8888,
                frame.len() / size_of::<ColorRGBA8888>(),
            )
        };

        let frame_x_start = point_1.get_x() as usize;
        let frame_y_start = point_1.get_y() as usize;
        let width = (point_2.get_x() - point_1.get_x() + 1) as usize;
        let height = (point_2.get_y() - point_1.get_y() + 1) as usize;

        for (y, data_row) in data_buffer.chunks(width).enumerate().take(height) {
            let frame_row_start = (frame_y_start + y) * frame_width + frame_x_start;
            let frame_row_start = usize::min(frame_row_start, frame.len());
            let frame_row_end = usize::min(frame_row_start + width, frame.len());
            let frame_row = &mut frame[frame_row_start..frame_row_end];

            frame_row
                .iter_mut()
                .zip(data_row.iter())
                .for_each(|(destination, &source)| {
                    *destination = source.into();
                });
        }

        Ok(())
    }
}
