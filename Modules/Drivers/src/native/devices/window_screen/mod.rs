mod keyboard;
mod pointer;
mod screen;
mod window;
mod wrapper;

use std::sync::{Arc, Mutex};

use file_system::{create_device, Device_type};
use graphics::Point_type;

use keyboard::*;
use pointer::*;
use screen::*;
use window::*;
use wrapper::*;

pub fn new(resolution: Point_type) -> Result<(Device_type, Device_type, Device_type), String> {
    let inner = Arc::new(Mutex::new(Inner_type::new(resolution)?));

    let screen_device = Screen_device_type::new(inner.clone());

    let pointer_device = Pointer_device_type::new(inner.clone());

    let keyboard_device = Keyboard_device_type::new(inner);

    Ok((
        create_device!(screen_device),
        create_device!(pointer_device),
        create_device!(keyboard_device),
    ))
}
