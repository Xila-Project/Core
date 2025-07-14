mod keyboard;
mod pointer;
mod screen;
mod window;
mod wrapper;

use std::sync::{Arc, Mutex};

use file_system::{create_device, Device};
use graphics::Point;

use keyboard::*;
use pointer::*;
use screen::*;
use window::*;
use wrapper::*;

pub fn new(resolution: Point) -> Result<(Device, Device, Device), String> {
    let inner = Arc::new(Mutex::new(Inner::new(resolution)?));

    let screen_device = ScreenDevice::new(inner.clone());

    let pointer_device = PointerDevice::new(inner.clone());

    let keyboard_device = KeyboardDevice::new(inner);

    Ok((
        create_device!(screen_device),
        create_device!(pointer_device),
        create_device!(keyboard_device),
    ))
}
