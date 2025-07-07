#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

extern crate alloc;

mod Area;
mod Color;
mod Display;
mod Draw_buffer;
mod Error;
mod Event;
mod Input;
mod Manager;
mod Point;
mod Screen;
mod Window;

pub mod LVGL;

pub use Area::*;
pub use Color::*;
pub use Display::*;
pub use Draw_buffer::*;
pub use Error::*;
pub use Event::*;
pub use Input::*;
pub use Manager::*;
pub use Point::*;
pub use Screen::*;
pub use Window::*;
