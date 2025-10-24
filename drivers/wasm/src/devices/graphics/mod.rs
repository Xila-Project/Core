mod canvas_screen;
mod full_screen_canvas;
mod keyboard;
mod mouse;

use alloc::string::String;
pub use canvas_screen::*;
use file_system::{Device, create_device};
use graphics::Point;
pub use keyboard::*;
pub use mouse::*;
use web_sys::{HtmlCanvasElement, Window};

pub struct GraphicsDevices {
    pub screen_device: Device,
    pub mouse_device: Device,
    pub keyboard_device: Device,
    pub canvas: HtmlCanvasElement,
}

pub async fn new() -> Result<GraphicsDevices, String> {
    let canvas = full_screen_canvas::new()?;

    let screen_device = CanvasScreenDevice::new(canvas.clone()).await?;

    let keyboard_device = KeyboardDevice::new()?;

    let mouse_device = MouseDevice::new(&canvas)?;

    Ok(GraphicsDevices {
        screen_device: create_device!(screen_device),
        mouse_device: create_device!(mouse_device),
        keyboard_device: create_device!(keyboard_device),
        canvas,
    })
}

fn get_window_size(window: &Window) -> Option<(u32, u32)> {
    let width = window.inner_width().ok()?.as_f64()? as u32;
    let height = window.inner_height().ok()?.as_f64()? as u32;
    Some((width, height))
}

pub fn get_resolution() -> Result<Point, &'static str> {
    let window = web_sys::window().ok_or("Failed to get window")?;

    let (width, height) = get_window_size(&window).ok_or("Failed to get window size")?;

    Ok(Point::new(width as _, height as _))
}
