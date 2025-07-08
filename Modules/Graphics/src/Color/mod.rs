mod argb8888;
pub mod palette;
mod rgb565;
mod rgb888;
mod rgba8888;

pub use argb8888::*;
pub use rgb565::*;
pub use rgb888::*;
pub use rgba8888::*;

pub type Color_type = Color_RGB888_type;

pub type Rendering_color_type = Color_RGB565_type;
