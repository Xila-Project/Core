mod keyboard;
mod pointer;
mod screen;
mod window;
mod wrapper;

use std::sync::{Arc, Mutex};

use file_system::{create_device, DeviceType};
use graphics::Point;

use keyboard::*;
use pointer::*;
use screen::*;
use window::*;
use wrapper::*;

pub fn new(resolution: Point) -> Result<(DeviceType, DeviceType, DeviceType), String> {
    let inner = Arc::new(Mutex::new(Inner::new(resolution)?));

    let screen_device = ScreenDeviceType::new(inner.clone());

    let pointer_device = PointerDeviceType::new(inner.clone());

    let keyboard_device = KeyboardDeviceType::new(inner);

    Ok((
        create_device!(screen_device),
        create_device!(pointer_device),
        create_device!(keyboard_device),
    ))
}
