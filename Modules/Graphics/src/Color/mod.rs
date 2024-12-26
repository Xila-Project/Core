mod ARGB8888;
pub mod Palette;
mod RGB565;
mod RGB888;
mod RGBA8888;

pub use ARGB8888::*;
pub use RGB565::*;
pub use RGB888::*;
pub use RGBA8888::*;

pub type Color_type = Color_RGB888_type;

pub type Rendering_color_type = Color_RGB565_type;
