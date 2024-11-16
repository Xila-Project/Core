#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Area;
mod Color;
mod Display;
mod Draw_buffer;
mod Error;
mod Input;
mod Manager;
mod Point;
mod Pointer;
mod Screen;
mod Window;

pub use Area::*;
pub use Color::*;
pub use Display::*;
pub use Draw_buffer::*;
pub use Error::*;
pub use Input::*;
pub use Manager::*;
pub use Point::*;
pub use Pointer::*;
pub use Screen::*;
pub use Window::*;

pub use lvgl_rust_sys as lvgl;
