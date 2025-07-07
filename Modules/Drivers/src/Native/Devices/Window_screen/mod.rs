mod Keyboard;
mod Pointer;
mod Screen;
mod Window;
mod Wrapper;

use std::sync::{Arc, Mutex};

use File_system::{Create_device, Device_type};
use Graphics::Point_type;

use Keyboard::*;
use Pointer::*;
use Screen::*;
use Window::*;
use Wrapper::*;

pub fn New(Resolution: Point_type) -> Result<(Device_type, Device_type, Device_type), String> {
    let inner = Arc::new(Mutex::new(Inner_type::new(Resolution)?));

    let Screen_device = Screen_device_type::new(inner.clone());

    let Pointer_device = Pointer_device_type::new(inner.clone());

    let Keyboard_device = Keyboard_device_type::new(inner);

    Ok((
        Create_device!(Screen_device),
        Create_device!(Pointer_device),
        Create_device!(Keyboard_device),
    ))
}
