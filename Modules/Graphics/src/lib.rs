#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Display;
mod Draw_buffer;
mod Error;
mod Input;
mod Manager;
//mod Window;

pub use Display::*;
pub use Draw_buffer::*;
pub use Error::*;
pub use Input::*;
pub use Manager::*;
//pub use Window::*;

pub use lvgl;
pub use lvgl::sys;

#[cfg(feature = "ARGB8888")]
type Color_type = Screen::Color_ARGB8888_type;

#[cfg(feature = "RGB565")]
type Color_type = Screen::Color_RGB565_type;
